'use strict';

const debug = require('debug')('joystream:runtime:assets');

const { Null, U64 } = require('@polkadot/types/primitive');

const { _ } = require('lodash');

const { decodeAddress } = require('@polkadot/keyring');

function parseContentId(contentId) {
  try {
    return decodeAddress(contentId)
  } catch (err) {
    return contentId
  }
}

/*
 * Add asset related functionality to the substrate API.
 */
class AssetsApi
{
  static async create(base)
  {
    const ret = new AssetsApi();
    ret.base = base;
    await ret.init();
    return ret;
  }

  async init(account_file)
  {
    debug('Init');
  }

  /*
   * Create a data object.
   */
  async createDataObject(accountId, contentId, doTypeId, size)
  {
    contentId = parseContentId(contentId)
    const tx = this.base.api.tx.dataDirectory.addContent(contentId, doTypeId, size);
    await this.base.signAndSendWithRetry(accountId, tx);

    // If the data object constructed properly, we should now be able to return
    // the data object from the state.
    return await this.getDataObject(contentId);
  }

  /*
   * Return the Data Object for a CID
   */
  async getDataObject(contentId)
  {
    contentId = parseContentId(contentId)
    const obj = await this.base.api.query.dataDirectory.dataObjectByContentId(contentId);
    return obj;
  }

  /*
   * Verify the liaison state for a DO:
   * - Check the content ID has a DO
   * - Check the account is the liaison
   * - Check the liaison state is pending
   *
   * Each failure errors out, success returns the data object.
   */
  async checkLiaisonForDataObject(accountId, contentId)
  {
    contentId = parseContentId(contentId)
    let obj = await this.getDataObject(contentId);
    if (obj.isNone) {
      throw new Error(`No DataObject created for content ID: ${contentId}`);
    }

    const encode = require('@polkadot/keyring/address/encode').default;
    const encoded = encode(obj.raw.liaison);
    if (encoded != accountId) {
      throw new Error(`This storage node is not liaison for the content ID: ${contentId}`);
    }

    if (_.isEqual(obj.raw.liaison_judgement, new Null())) {
      throw new Error('Internal error; liaison_judgement should always be set!');
    }

    const judge_val = obj.raw.liaison_judgement.raw;
    const judge_arr = obj.raw.liaison_judgement._enum;

    if (judge_arr[judge_val] != 'Pending') {
      throw new Error(`Expected Pending judgement, but found: ${judge_arr[judge_val]}`);
    }

    return obj.unwrap();
  }

  /*
   * Changes a data object liaison judgement.
   */
  async acceptContent(accountId, contentId)
  {
    contentId = parseContentId(contentId)
    const tx = this.base.api.tx.dataDirectory.acceptContent(contentId);
    return await this.base.signAndSendWithRetry(accountId, tx);
  }

  /*
   * Changes a data object liaison judgement.
   */
  async rejectContent(accountId, contentId)
  {
    contentId = parseContentId(contentId)
    const tx = this.base.api.tx.dataDirectory.rejectContent(contentId);
    return await this.base.signAndSendWithRetry(accountId, tx);
  }

  /*
   * Create storage relationship
   */
  async createStorageRelationship(accountId, contentId, callback)
  {
    contentId = parseContentId(contentId)
    const tx = this.base.api.tx.dataObjectStorageRegistry.addRelationship(contentId);

    const subscribed = [['dataObjectStorageRegistry', 'DataObjectStorageRelationshipAdded']];
    return await this.base.signAndSendWithRetry(accountId, tx, 3, subscribed, callback);
  }

  /*
   * Get storage relationship for contentId
   */
  async getStorageRelationshipAndId(accountId, contentId) {
    contentId = parseContentId(contentId)
    let rids = await this.base.api.query.dataObjectStorageRegistry.relationshipsByContentId(contentId);

    while(rids.length) {
      const relationshipId = rids.shift();
      let relationship = await this.base.api.query.dataObjectStorageRegistry.relationships(relationshipId);
      relationship = relationship.unwrap();
      if (relationship.storage_provider.eq(decodeAddress(accountId))) {
        return ({ relationship, relationshipId });
      }
    }

    return {};
  }

  async createAndReturnStorageRelationship(accountId, contentId)
  {
    contentId = parseContentId(contentId)
    return new Promise(async (resolve, reject) => {
      try {
        await this.createStorageRelationship(accountId, contentId, (events) => {
          events.forEach((event) => {
            resolve(event[1].DataObjectStorageRelationshipId);
          });
        });
      } catch (err) {
        reject(err);
      }
    });
  }

  /*
   * Toggle ready state for DOSR.
   */
  async toggleStorageRelationshipReady(accountId, dosrId, ready)
  {
    var tx = ready
      ? this.base.api.tx.dataObjectStorageRegistry.setRelationshipReady(dosrId)
      : this.base.api.tx.dataObjectStorageRegistry.unsetRelationshipReady(dosrId);
    return await this.base.signAndSendWithRetry(accountId, tx);
  }

  async getKnownContentIds() {
    return this.base.api.query.dataDirectory.knownContentIds();
  }
}

module.exports = {
  AssetsApi: AssetsApi,
}
