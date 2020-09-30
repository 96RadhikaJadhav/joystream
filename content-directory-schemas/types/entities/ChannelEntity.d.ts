/* tslint:disable */
/**
 * This file was automatically generated by json-schema-to-typescript.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run json-schema-to-typescript to regenerate this file.
 */

/**
 * JSON schema for reference to Language entity based on runtime schema
 */
export type LanguageReference =
  | {
      /**
       * ISO 639-1 code of the language (ie. en)
       */
      Code: string
    }
  | number

/**
 * JSON schema for entities based on Channel runtime schema
 */
export interface ChannelEntity {
  /**
   * The title of the Channel
   */
  title: string
  /**
   * The description of a Channel
   */
  description: string
  /**
   * Url for Channel's cover (background) photo. Recommended ratio: 16:9.
   */
  coverPhotoUrl: string
  /**
   * Channel's avatar photo.
   */
  avatarPhotoURL: string
  /**
   * Flag signaling whether a channel is public.
   */
  isPublic: boolean
  /**
   * Flag signaling whether a channel is curated/verified.
   */
  isCurated: boolean
  /**
   * The primary langauge of the channel's content
   */
  language?:
    | {
        new: LanguageEntity
      }
    | {
        existing: LanguageReference
      }
}
/**
 * JSON schema for entities based on Language runtime schema
 */
export interface LanguageEntity {
  /**
   * The name of the language (ie. English)
   */
  Name: string
  /**
   * ISO 639-1 code of the language (ie. en)
   */
  Code: string
}
