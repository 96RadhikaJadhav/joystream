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
// const readline = require('readline');

const debug = require('debug')('joystream:runtime:identities');

const { Keyring } = require('@polkadot/keyring');
// const { Null } = require('@polkadot/types/primitive');
const util_crypto = require('@polkadot/util-crypto');

// const { _ } = require('lodash');

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
    this.keyring = new Keyring();

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
    debug('Successfully initialized with address', key.address);
    return key;
  }

  /*
   * Try to unlock a key if it isn't already unlocked.
   * passphrase should be supplied as argument.
   */
  async tryUnlock(key, passphrase)
  {
    if (!key.isLocked) {
      debug('Key is not locked, not attempting to unlock')
      return;
    }

    // First try with an empty passphrase - for convenience
    try {
      key.decodePkcs8('');

      if (passphrase) {
        debug('Key was not encrypted, supplied passphrase was ignored');
      }

      return;
    } catch (err) {
      // pass
    }

    // Then with supplied passphrase
    try {
      debug('Decrypting with supplied passphrase');
      key.decodePkcs8(passphrase);
      return;
    } catch (err) {
      // pass
    }

    // If that didn't work, ask for a passphrase if appropriate
    if (this.canPromptForPassphrase) {
      passphrase = await this.askForPassphrase(key.address);
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
    const memberIds = await this.memberIdsOf(accountId); // return array of member ids
    return memberIds.length > 0 // true if at least one member id exists for the acccount
  }

  /*
   * Return the member IDs of an account
   */
  async memberIdsOf(accountId)
  {
    const decoded = this.keyring.decodeAddress(accountId);
    return await this.base.api.query.members.memberIdsByRootAccountId(decoded);
  }

  /*
   * Return the first member ID of an account, or undefined if not a member.
   */
  async firstMemberIdOf(accountId)
  {
    const decoded = this.keyring.decodeAddress(accountId);
    let ids = await this.base.api.query.members.memberIdsByRootAccountId(decoded);
    return ids[0]
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
