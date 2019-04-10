'use strict';

const debug = require('debug')('joystream:substrate:roles');

const { Null, U64 } = require('@polkadot/types/primitive');

const { _ } = require('lodash');

const { BalancesApi } = require('joystream/substrate/balances');

const ROLE_STORAGE = new U64(0x00);

/*
 * Add role related functionality to the substrate API.
 */
class RolesApi extends BalancesApi
{
  static async create(account_file)
  {
    const ret = new RolesApi();
    await ret.init(account_file);
    return ret;
  }

  async init(account_file)
  {
    debug('Init');

    // Super init
    await super.init(account_file);
  }

  /*
   * Raises errors if the given account ID is not valid for staking as the given
   * role. The role should be one of the ROLE_* constants above.
   */
  async checkAccountForStaking(accountId, role)
  {
    role = role || ROLE_STORAGE;

    if (!await this.isMember(accountId)) {
      const msg = `Account with id "${accountId}" is not a member!`;
      debug(msg);
      throw new Error(msg);
    }

    if (!await this.hasBalanceForRoleStaking(accountId, role)) {
      const msg = `Account with id "${accountId}" does not have sufficient free balance for role staking!`;
      debug(msg);
      throw new Error(msg);
    }

    debug(`Account with id "${accountId}" is a member with sufficient free balance, able to proceed.`);
    return true;
  }

  /*
   * Returns the required balance for staking for a role.
   */
  async requiredBalanceForRoleStaking(role)
  {
    const params = await this.api.query.actors.parameters(role);
    if (_.isEqual(params.raw, new Null())) {
      throw new Error(`Role ${role} is not defined!`);
    }
    const result = params.raw.min_stake.add(params.raw.entry_request_fee);
    return result;
  }

  /*
   * Returns true/false if the given account has the balance required for
   * staking for the given role.
   */
  async hasBalanceForRoleStaking(accountId, role)
  {
    const required = await this.requiredBalanceForRoleStaking(role);
    return await this.hasMinimumBalanceOf(accountId, required);
  }

  /*
   * Transfer enough funds to allow the recipient to stake for the given role.
   */
  async transferForStaking(from, to, role)
  {
    const required = await this.requiredBalanceForRoleStaking(role);
    return await this.transfer(from, to, required);
  }

  /*
   * Send a role application.
   * - The role account must not be a member, but have sufficient funds for
   *   staking.
   * - The member account must be a member.
   *
   * After sending this application, the member account will have role request
   * in the 'My Requests' tab of the app.
   */
  async applyForRole(roleAccountId, role, memberAccountId)
  {
    const memberId = await this.memberIdOf(memberAccountId);
    if (_.isEqual(memberId.raw, new Null())) {
      throw new Error('Account is not a member!');
    }
    const converted = memberId.raw;

    const from_key = this.keyring.getPair(roleAccountId);
    if (from_key.isLocked()) {
      throw new Error('Must unlock key before using it to sign!');
    }

    return await this.api.tx.actors.roleEntryRequest(role, converted)
      .signAndSend(from_key);
  }

  /*
   * Check whether the given role is occupying the given role.
   */
  async checkForRole(roleAccountId, role)
  {
    const actor = await this.api.query.actors.actorByAccountId(roleAccountId);
    return !_.isEqual(actor.raw, new Null());
  }

  /*
   * Same as checkForRole(), but if the account is not currently occupying the
   * role, wait for the appropriate `actors.Staked` event to be emitted.
   */
  async waitForRole(roleAccountId, role)
  {
    if (await this.checkForRole(roleAccountId, role)) {
      return true;
    }

    return new Promise((resolve, reject) => {
      this.waitForEvent('actors', 'Staked').then((values) => {
        const name = values[0];
        const payload = values[1];

        if (payload.AccountId == roleAccountId) {
          resolve(true);
        }
      });
    });
  }
}

module.exports = {
  RolesApi: RolesApi,
  ROLE_STORAGE: ROLE_STORAGE,
}
