import { ApiPromise, WsProvider } from '@polkadot/api'
import { types as joyTypes } from '@joystream/types'
import { Keyring } from '@polkadot/keyring'
// Import input parser and channel entity from cd-schemas (we use it as library here)
import { InputParser } from 'cd-schemas'
import { ChannelEntity } from 'cd-schemas/types/entities/ChannelEntity'

async function main() {
  // Initialize the api
  const provider = new WsProvider('ws://127.0.0.1:9944')
  const api = await ApiPromise.create({ provider, types: joyTypes })

  // Get Alice keypair
  const keyring = new Keyring()
  keyring.addFromUri('//Alice', undefined, 'sr25519')
  const [ALICE] = keyring.getPairs()

  // Create partial channel entity, only containing the fields we wish to update
  const channelUpdateInput: Partial<ChannelEntity> = {
    title: 'Updated channel title',
  }

  // Create the parser with known entity schemas (the ones in content-directory-schemas/inputs)
  const parser = InputParser.createWithKnownSchemas(api)

  // We can reuse InputParser's `findEntityIdByUniqueQuery` method to find entityId of the channel we
  // created in ./createChannel.ts example (normally we would probably use some other way to do it, ie.: query node)
  const CHANNEL_ID = await parser.finidEntityIdByUniqueQuery({ title: 'Example channel' }, 'Channel')

  // Use createEntityUpdateOperation to parse the input
  const updateOperation = await parser.createEntityUpdateOperation(
    channelUpdateInput,
    'Channel', // Class name
    CHANNEL_ID // Id of the entity we want to update
  )

  await api.tx.contentDirectory
    .transaction(
      { Member: 0 }, // We use member with id 0 as actor (in this case we assume this is Alice)
      [updateOperation] // The only operation we execute in this transaction is a single updateOperation
    )
    .signAndSend(ALICE)
}

main()
  .then(() => process.exit())
  .catch(console.error)
