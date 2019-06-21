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
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

'use strict';

const path = require('path');
const fs = require('fs');
const readline = require('readline');

const debug = require('debug')('joystream:runtime:identities');

const { Keyring } = require('@joystream/crypto');
const { Null } = require('@polkadot/types/primitive');
const util_crypto = require('@polkadot/util-crypto');

const { _ } = require('lodash');

/*
 * Add identity management to the substrate API.
 *
 * This loosely groups: accounts, key management, and membership.
 */
class IdentitiesApi
{
  static async create(base, {account_file, passphrase, canPromptForPassphrase})
  {
    const ret = new IdentitiesApi();
    ret.base = base;
    await ret.init(account_file, passphrase, canPromptForPassphrase);
    return ret;
  }

  async init(account_file, passphrase, canPromptForPassphrase)
  {
    debug('Init');

    // Creatre keyring
    this.keyring = await Keyring.create();

    this.canPromptForPassphrase = canPromptForPassphrase || false;

    // Load account file, if possible.
    try {
      this.key = await this.loadUnlock(account_file, passphrase);
    } catch (err) {
      debug('Error loading account file:', err.message);
    }
  }

  /*
   * Load a key file and unlock it if necessary.
   */
  async loadUnlock(account_file, passphrase)
  {
    const fullname = path.resolve(account_file);
    debug('Initializing key from', fullname);
    const key = this.keyring.addFromJson(require(fullname));
    await this.tryUnlock(key, passphrase);
    debug('Successfully initialized with address', key.address());
    return key;
  }

  /*
   * Try to unlock a key if it isn't already unlocked.
   * passphrase should be supplied as argument.
   */
  async tryUnlock(key, passphrase)
  {
    if (!key.isLocked()) {
      return;
    }

    // First try with an empty passphrase - for convenience
    try {
      key.decodePkcs8('');
      return;
    } catch (err) {
      // pass
    }

    // Then with supplied passphrase
    try {
      console.log('trying with passphrase:', passphrase)
      key.decodePkcs8(passphrase);
      return;
    } catch (err) {
      // pass
    }

    // If that didn't work, ask for a passphrase if appropriate
    if (this.canPromptForPassphrase) {
      passphrase = await this.askForPassphrase(key.address());
      key.decodePkcs8(passphrase);
      return
    }

    throw new Error('invalid passphrase supplied');
  }

  /*
   * Ask for a passphrase
   */
  askForPassphrase(address)
  {
    // Query for passphrase
    const prompt = require('password-prompt');
    return prompt(`Enter passphrase for ${address}: `, { required: false });
  }

  /*
   * Return true if the account is a member
   */
  async isMember(accountId)
  {
    const memberId = await this.memberIdOf(accountId);
    return !_.isEqual(memberId.raw, new Null());
  }

  /*
   * Return true if the account is an actor/role account
   */
  async isActor(accountId)
  {
    const decoded = this.keyring.decodeAddress(accountId);
    const actor = await this.base.api.query.actors.actorByAccountId(decoded)
    return actor.isSome
  }

  /*
   * Return the member ID of an account - this is an Option, so '.raw' may or may not
   * have a useful value.
   */
  async memberIdOf(accountId)
  {
    const decoded = this.keyring.decodeAddress(accountId);
    return await this.base.api.query.membership.memberIdByAccountId(decoded);
  }

  /*
   * Create a new key for the given role *name*. If no name is given,
   * default to 'storage'.
   */
  async createRoleKey(accountId, role)
  {
    role = role || 'storage';

    // Generate new key pair
    const keyPair = util_crypto.naclKeypairFromRandom();

    // Encode to an address.
    const addr = this.keyring.encodeAddress(keyPair.publicKey);
    debug('Generated new key pair with address', addr);

    // Add to key wring. We set the meta to identify the account as
    // a role key.
    const meta = {
      name: `${role} role account for ${accountId}`,
    };

    const createPair = require('@polkadot/keyring/pair').default;
    const pair = createPair('ed25519', keyPair, meta);

    this.keyring.addPair(pair);

    return pair;
  }

  /*
   * Export a key pair to JSON. Will ask for a passphrase.
   */
  async exportKeyPair(accountId)
  {
    const passphrase = await this.askForPassphrase(accountId);

    // Produce JSON output
    return this.keyring.toJson(accountId, passphrase);
  }

  /*
   * Export a key pair and write it to a JSON file with the account ID as the
   * name.
   */
  async writeKeyPairExport(accountId, prefix)
  {
    // Generate JSON
    const data = await this.exportKeyPair(accountId);

    // Write JSON
    var filename = `${data.address}.json`;
    if (prefix) {
      const path = require('path');
      filename = path.resolve(prefix, filename);
    }
    fs.writeFileSync(filename, JSON.stringify(data), {
      encoding: 'utf8',
      mode: 0o600,
    });

    return filename;
  }
}

module.exports = {
  IdentitiesApi: IdentitiesApi,
}
