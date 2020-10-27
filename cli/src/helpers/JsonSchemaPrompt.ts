import Ajv from 'ajv'
import inquirer, { DistinctQuestion } from 'inquirer'
import _ from 'lodash'
import RefParser, { JSONSchema } from '@apidevtools/json-schema-ref-parser'
import chalk from 'chalk'
import { BOOL_PROMPT_OPTIONS } from './prompting'
import { getSchemasLocation } from 'cd-schemas'
import path from 'path'

type CustomPromptMethod = () => Promise<any>
type CustomPrompt = DistinctQuestion | CustomPromptMethod | { $item: CustomPrompt }

// For the explaination of "string & { x: never }", see: https://github.com/microsoft/TypeScript/issues/29729
// eslint-disable-next-line @typescript-eslint/ban-types
export type JsonSchemaCustomPrompts<T = Record<string, unknown>> = [keyof T | (string & {}) | RegExp, CustomPrompt][]

// Default schema path for resolving refs
// TODO: Would be nice to skip the filename part (but without it it doesn't work)
const DEFAULT_SCHEMA_PATH = getSchemasLocation('entities') + path.sep

export class JsonSchemaPrompter<JsonResult> {
  schema: JSONSchema
  schemaPath: string
  customPropmpts?: JsonSchemaCustomPrompts
  ajv: Ajv.Ajv
  filledObject: Partial<JsonResult>

  constructor(
    schema: JSONSchema,
    defaults?: Partial<JsonResult>,
    customPrompts?: JsonSchemaCustomPrompts,
    schemaPath: string = DEFAULT_SCHEMA_PATH
  ) {
    this.customPropmpts = customPrompts
    this.schema = schema
    this.schemaPath = schemaPath
    // allErrors prevents .validate from setting only one error when in fact there are multiple
    this.ajv = new Ajv({ allErrors: true })
    this.filledObject = defaults || {}
  }

  private oneOfToChoices(oneOf: JSONSchema[]) {
    const choices: { name: string; value: number | string }[] = []

    oneOf.forEach((pSchema, index) => {
      if (pSchema.description) {
        choices.push({ name: pSchema.description, value: index })
      } else if (pSchema.type === 'object' && pSchema.properties) {
        choices.push({ name: `{ ${Object.keys(pSchema.properties).join(', ')} }`, value: index })
      } else {
        choices.push({ name: index.toString(), value: index })
      }
    })

    return choices
  }

  private getCustomPrompt(propertyPath: string): CustomPrompt | undefined {
    const found = this.customPropmpts?.find(([pathToMatch]) =>
      pathToMatch instanceof RegExp ? pathToMatch.test(propertyPath) : propertyPath === pathToMatch
    )

    return found ? found[1] : undefined
  }

  private propertyDisplayName(propertyPath: string) {
    return chalk.green(propertyPath)
  }

  private async prompt(schema: JSONSchema, propertyPath = '', custom?: CustomPrompt): Promise<any> {
    const customPrompt: CustomPrompt | undefined = custom || this.getCustomPrompt(propertyPath)
    const propDisplayName = this.propertyDisplayName(propertyPath)

    // Custom prompt
    if (typeof customPrompt === 'function') {
      return await this.promptWithRetry(customPrompt, propertyPath, true)
    }

    // oneOf
    if (schema.oneOf) {
      const oneOf = schema.oneOf as JSONSchema[]
      const choices = this.oneOfToChoices(oneOf)
      const { choosen } = await inquirer.prompt({ name: 'choosen', message: propDisplayName, type: 'list', choices })
      return await this.prompt(oneOf[choosen], propertyPath)
    }

    // object
    if (schema.type === 'object' && schema.properties) {
      const value: Record<string, any> = {}
      for (const [pName, pSchema] of Object.entries(schema.properties)) {
        value[pName] = await this.prompt(pSchema, propertyPath ? `${propertyPath}.${pName}` : pName)
      }
      return value
    }

    // array
    if (schema.type === 'array' && schema.items) {
      return await this.promptWithRetry(() => this.promptArray(schema, propertyPath), propertyPath, true)
    }

    // "primitive" values:
    const currentValue = _.get(this.filledObject, propertyPath)
    const basicPromptOptions: DistinctQuestion = {
      message: propDisplayName,
      default: currentValue !== undefined ? currentValue : schema.default,
    }

    let additionalPromptOptions: DistinctQuestion | undefined
    let normalizer: (v: any) => any = (v) => v

    // Prompt options
    if (schema.enum) {
      additionalPromptOptions = { type: 'list', choices: schema.enum as any[] }
    } else if (schema.type === 'boolean') {
      additionalPromptOptions = BOOL_PROMPT_OPTIONS
    }

    // Normalizers
    if (schema.type === 'integer') {
      normalizer = (v) => (parseInt(v).toString() === v ? parseInt(v) : v)
    }

    if (schema.type === 'number') {
      normalizer = (v) => (Number(v).toString() === v ? Number(v) : v)
    }

    const promptOptions = { ...basicPromptOptions, ...additionalPromptOptions, ...customPrompt }
    // Need to wrap in retry, because "validate" will not get called if "type" is "list" etc.
    return await this.promptWithRetry(
      async () => normalizer(await this.promptSimple(promptOptions, propertyPath, normalizer)),
      propertyPath
    )
  }

  private setValueAndGetError(propertyPath: string, value: any, nestedErrors = false): string | null {
    _.set(this.filledObject as Record<string, unknown>, propertyPath, value)
    this.ajv.validate(this.schema, this.filledObject) as boolean
    return this.ajv.errors
      ? this.ajv.errors
          .filter((e) => (nestedErrors ? e.dataPath.startsWith(`.${propertyPath}`) : e.dataPath === `.${propertyPath}`))
          .map((e) => (e.dataPath.replace(`.${propertyPath}`, '') || 'This value') + ` ${e.message}`)
          .join(', ')
      : null
  }

  private async promptArray(schema: JSONSchema, propertyPath: string) {
    if (!schema.items) {
      return []
    }
    const { maxItems = Number.MAX_SAFE_INTEGER } = schema
    let currItem = 0
    const result = []
    while (currItem < maxItems) {
      const { next } = await inquirer.prompt([
        {
          ...BOOL_PROMPT_OPTIONS,
          name: 'next',
          message: `Do you want to add another item to ${this.propertyDisplayName(propertyPath)} array?`,
        },
      ])
      if (!next) {
        break
      }
      const itemSchema = Array.isArray(schema.items) ? schema.items[schema.items.length % currItem] : schema.items
      result.push(await this.prompt(typeof itemSchema === 'boolean' ? {} : itemSchema, `${propertyPath}[${currItem}]`))

      ++currItem
    }

    return result
  }

  private async promptSimple(promptOptions: DistinctQuestion, propertyPath: string, normalize?: (v: any) => any) {
    const { result } = await inquirer.prompt([
      {
        ...promptOptions,
        name: 'result',
        validate: (v) => {
          v = normalize ? normalize(v) : v
          return (
            this.setValueAndGetError(propertyPath, v) ||
            (promptOptions.validate ? promptOptions.validate(v) : true) ||
            true
          )
        },
      },
    ])

    return result
  }

  private async promptWithRetry(customMethod: CustomPromptMethod, propertyPath: string, nestedErrors = false) {
    let error: string | null
    let value: any
    do {
      value = await customMethod()
      error = this.setValueAndGetError(propertyPath, value, nestedErrors)
      if (error) {
        console.log('\n')
        console.log('Provided value:', value)
        console.warn(`ERROR: ${error}`)
        console.warn(`Try providing the input for ${propertyPath} again...`)
      }
    } while (error)

    return value
  }

  async getMainSchema() {
    return await RefParser.dereference(this.schemaPath, this.schema, {})
  }

  async promptAll() {
    await this.prompt(await this.getMainSchema())
    return this.filledObject as JsonResult
  }

  async promptMultipleProps<P extends keyof JsonResult & string, PA extends readonly P[]>(
    props: PA
  ): Promise<{ [K in PA[number]]: Exclude<JsonResult[K], undefined> }> {
    const result: Partial<{ [K in PA[number]]: Exclude<JsonResult[K], undefined> }> = {}
    for (const prop of props) {
      result[prop] = await this.promptSingleProp(prop)
    }

    return result as { [K in PA[number]]: Exclude<JsonResult[K], undefined> }
  }

  async promptSingleProp<P extends keyof JsonResult & string>(
    p: P,
    customPrompt?: CustomPrompt
  ): Promise<Exclude<JsonResult[P], undefined>> {
    const mainSchema = await this.getMainSchema()
    await this.prompt(mainSchema.properties![p] as JSONSchema, p, customPrompt)
    return this.filledObject[p] as Exclude<JsonResult[P], undefined>
  }
}
