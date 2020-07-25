/*
 * This file is part of the storage node for the Joystream project.
 * Copyright (C) 2019 Joystream Contributors
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

'use strict'

const debug = require('debug')('joystream:runtime:base')

const { registerJoystreamTypes } = require('@joystream/types')
const { ApiPromise, WsProvider } = require('@polkadot/api')
const { IdentitiesApi } = require('@joystream/storage-runtime-api/identities')
const { BalancesApi } = require('@joystream/storage-runtime-api/balances')
const { WorkersApi } = require('@joystream/storage-runtime-api/workers')
const { AssetsApi } = require('@joystream/storage-runtime-api/assets')
const { DiscoveryApi } = require('@joystream/storage-runtime-api/discovery')
const { SystemApi } = require('@joystream/storage-runtime-api/system')
const AsyncLock = require('async-lock')
const Promise = require('bluebird')

Promise.config({
  cancellation: true,
})

// const ASYNC_LOCK_TIMEOUT = 30 * 1000
const TX_TIMEOUT = 20 * 1000

/*
 * Initialize runtime (substrate) API and keyring.
 */
class RuntimeApi {
  static async create(options) {
    const runtimeApi = new RuntimeApi()
    await runtimeApi.init(options || {})
    return runtimeApi
  }

  async init(options) {
    debug('Init')

    options = options || {}

    // Register joystream types
    registerJoystreamTypes()

    const provider = new WsProvider(options.provider_url || 'ws://localhost:9944')

    // Create the API instrance
    this.api = await ApiPromise.create({ provider })

    // this.asyncLock = new AsyncLock({ timeout: ASYNC_LOCK_TIMEOUT, maxPending: 100 })
    this.asyncLock = new AsyncLock()

    // Keep track locally of account nonces.
    this.nonces = {}

    // The storage provider id to use
    this.storageProviderId = parseInt(options.storageProviderId) // u64 instead ?

    // Ok, create individual APIs
    this.identities = await IdentitiesApi.create(this, {
      accountFile: options.account_file,
      passphrase: options.passphrase,
      canPromptForPassphrase: options.canPromptForPassphrase,
    })
    this.balances = await BalancesApi.create(this)
    this.workers = await WorkersApi.create(this)
    this.assets = await AssetsApi.create(this)
    this.discovery = await DiscoveryApi.create(this)
    this.system = await SystemApi.create(this)
  }

  disconnect() {
    this.api.disconnect()
  }

  executeWithAccountLock(accountId, func) {
    return this.asyncLock.acquire(`${accountId}`, func)
  }

  static matchingEvents(subscribed = [], events = []) {
    const filtered = events.filter((record) => {
      const { event } = record

      // Skip events we're not interested in.
      const matching = subscribed.filter((value) => {
        if (value[0] === '*' && value[1] === '*') {
          return true
        } else if (value[0] === '*') {
          return event.method === value[1]
        } else if (value[1] === '*') {
          return event.section === value[0]
        } else {
          return event.section === value[0] && event.method === value[1]
        }
      })
      return matching.length > 0
    })

    const mapped = filtered.map((record) => {
      const { event } = record
      const types = event.typeDef

      // FIX: we are loosing some items if they have the same type
      // only the first occurance is saved in the payload map. This is the cost of convenience
      // to get a value "by name" - why not just return the original EventRecord
      // and let the calller use the index to get the value desired?
      const payload = {}
      event.data.forEach((data, index) => {
        const type = types[index].type
        payload[type] = payload[type] || data
      })

      const fullName = `${event.section}.${event.method}`
      return [fullName, payload]
    })

    mapped.length && debug('Mapped Events', JSON.stringify(mapped))

    return mapped
  }

  /*
   * signAndSend() with nonce tracking, to enable concurrent sending of transacctions
   * so that they can be included in the same block. Allows you to use the accountId instead
   * of the key, without requiring an external Signer configured on the underlying ApiPromie
   *
   * If the subscribed events are given, and a callback as well, then the
   * callback is invoked with matching events.
   */
  async signAndSend(accountId, tx, subscribed) {
    // Accept both a string or AccountId as argument
    accountId = this.identities.keyring.encodeAddress(accountId)

    // Throws if keyPair is not found
    const fromKey = this.identities.keyring.getPair(accountId)

    // Key must be unlocked to use
    if (fromKey.isLocked) {
      throw new Error('Must unlock key before using it to sign!')
    }

    // Functions to be called when the submitted transaction is finalized. They are initialized
    // after the transaction is submitted to the resolve and reject function of the final promise
    // returned by signAndSend
    // on extrinsic success
    let onFinalizedSuccess
    // on extrinsic failure
    let onFinalizedFailed

    // Function assigned when transaction is successfully submitted. Invoking it ubsubscribes from
    // listening to tx status updates.
    let unsubscribe

    let lastTxUpdateResult

    const handleTxUpdates = (result) => {
      const { events = [], status } = result
      if (!result || !status) {
        return
      }

      lastTxUpdateResult = result

      // Deal with statuses which will prevent
      // extrinsic from finalizing.
      if (status.isUsurped) {
        debug(status.type)
        debug(JSON.stringify(status.asUsurped))
        onFinalizedFailed && onFinalizedFailed({ err: 'Usurped', result, tx: status.asFinalized })
      }

      if (status.isDropped) {
        debug(status.type)
        debug(JSON.stringify(status.asDropped))
        onFinalizedFailed && onFinalizedFailed({ err: 'Dropped', result, tx: status.asFinalized })
      }

      // My gutt says this comes before isReady and causes await send() to throw
      // and therefore onFinalizedFailed isn't initialized.
      // We don't need to do anything other than log it?
      if (status.isInvalid) {
        debug(status.type)
        debug(JSON.stringify(status.asInvalid))
        onFinalizedFailed && onFinalizedFailed({ err: 'Invalid', result, tx: status.asFinalized })
      }

      if (status.isFinalized) {
        const mappedEvents = RuntimeApi.matchingEvents(subscribed, events)
        const failed = result.findRecord('system', 'ExtrinsicFailed')
        const success = result.findRecord('system', 'ExtrinsicSuccess')
        const sudid = result.findRecord('sudo', 'Sudid')
        const sudoAsDone = result.findRecord('sudo', 'SudoAsDone')

        if (failed) {
          const {
            event: { data },
          } = failed
          const dispatchError = data[0]
          onFinalizedFailed({
            err: 'ExtrinsicFailed',
            mappedEvents,
            result,
            tx: status.asFinalized,
            dispatchError, // we get module number/id and index into the Error enum
          })
        } else if (success) {
          // Note: For root origin calls, the dispatch error is logged to console
          // we cannot get it in the events
          if (sudid) {
            const dispatchSuccess = sudid.event.data[0]
            if (dispatchSuccess.isTrue) {
              onFinalizedSuccess({ mappedEvents, result, tx: status.asFinalized })
            } else {
              onFinalizedFailed({ err: 'SudoFailed', mappedEvents, result, tx: status.asFinalized })
            }
          } else if (sudoAsDone) {
            const dispatchSuccess = sudoAsDone.event.data[0]
            if (dispatchSuccess.isTrue) {
              onFinalizedSuccess({ mappedEvents, result, tx: status.asFinalized })
            } else {
              onFinalizedFailed({ err: 'SudoAsFailed', mappedEvents, result, tx: status.asFinalized })
            }
          } else {
            onFinalizedSuccess({ mappedEvents, result, tx: status.asFinalized })
          }
        }
      }

      if (result.isCompleted) {
        unsubscribe()
      }
    }

    // synchronize access to nonce
    await this.executeWithAccountLock(accountId, async () => {
      const nonce = this.nonces[accountId] || (await this.api.query.system.accountNonce(accountId))

      try {
        unsubscribe = await tx.sign(fromKey, { nonce }).send(handleTxUpdates)
        debug('TransactionSubmitted')
        // transaction submitted successfully, increment and save nonce.
        this.nonces[accountId] = nonce.addn(1)
      } catch (err) {
        const errstr = err.toString()
        debug('TransactionRejected:', errstr)
        // This happens when nonce is already used in finalized transactions, ie. the selected nonce
        // was less than current account nonce. A few scenarios where this happens (other than incorrect code)
        // 1. When a past future tx got finalized because we submitted some transactions
        // using up the nonces upto that point.
        // 2. Can happen while storage-node is talkig to a joystream-node that is still not fully
        // synced.
        // 3. Storage role account holder sent a transaction just ahead of us via another app.
        if (errstr.indexOf('ExtrinsicStatus:: 1010: Invalid Transaction: Stale') !== -1) {
          // In case 1 or 3 a quick recovery could work by just incrementing, but since we
          // cannot detect which case we are in just reset and force re-reading nonce. Even
          // that may not be sufficient expect after a few more failures..
          delete this.nonces[accountId]
        }

        // Technically it means a transaction in the mempool with same
        // nonce and same fees being paid so we cannot replace it, either we didn't correctly
        // increment the nonce or someone external to this application sent a transaction
        // with same nonce ahead of us.
        if (errstr.indexOf('ExtrinsicStatus:: 1014: Priority is too low') !== -1) {
          delete this.nonces[accountId]
        }

        throw err
      }
    })

    // We cannot get tx updates for a future tx so return now to avoid blocking caller
    if (lastTxUpdateResult.status.isFuture) {
      debug('Warning: Submitted extrinsic with future nonce')
      return {}
    }

    // Return a promise that will resolve when the transaction finalizes.
    // On timeout it will be rejected. Timeout is a workaround for dealing with the
    // fact that if rpc connection is lost to node we have no way of detecting it or recovering.
    return new Promise((resolve, reject) => {
      onFinalizedSuccess = resolve
      onFinalizedFailed = reject
    }).timeout(TX_TIMEOUT)
  }

  /*
   * Sign and send a transaction expect event from
   * module and return eventProperty from the event.
   */
  async signAndSendThenGetEventResult(senderAccountId, tx, { eventModule, eventName, eventProperty }) {
    if (!eventModule || !eventName || !eventProperty) {
      throw new Error('MissingSubscribeEventDetails')
    }

    // event from a module,
    const subscribed = [[eventModule, eventName]]

    const { mappedEvents } = await this.signAndSend(senderAccountId, tx, subscribed)

    if (!mappedEvents) {
      // The tx was a future so it was not possible and will not be possible to get events
      throw new Error('NoEventsCanBeCaptured')
    }

    if (!mappedEvents.length) {
      // our expected event was not emitted
      throw new Error('ExpectedEventNotFound')
    }

    // fix - we may not necessarily want the first event
    // if there are multiple instances of the same event
    const firstEvent = mappedEvents[0]
    const payload = firstEvent[1]
    // Note if the event data contained more than one element of the same type
    // we can only get the first occurance
    return payload[eventProperty]
  }
}

module.exports = {
  RuntimeApi,
}
