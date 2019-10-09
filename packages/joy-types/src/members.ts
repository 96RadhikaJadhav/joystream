import { Enum, getTypeRegistry, Option, Struct, Null, bool, u64, u128, Text, GenericAccountId } from '@polkadot/types';
import { BlockNumber, Moment, BalanceOf } from '@polkadot/types/interfaces';
import { OptionText } from './index';
import AccountId from '@polkadot/types/primitive/Generic/AccountId';

export class MemberId extends u64 {}
export class PaidTermId extends u64 {}
export class SubscriptionId extends u64 {}

export class Paid extends PaidTermId {}
export class Screening extends GenericAccountId {}
export class Genesis extends Null {} // Works for enum variant that doesn't encapsulate any value?
export class EntryMethod extends Enum {
  constructor (value?: any, index?: number) {
    super({
      Paid,
      Screening,
      Genesis,
    }, value, index);
  }
}

export class Role extends Enum {
  constructor (value?: any) {
    super([
      'Storage'
    ], value);
  }
}

export type Profile = {
  handle: Text,
  avatar_uri: Text,
  about: Text,
  registered_at_block: BlockNumber,
  registered_at_time: Moment,
  entry: EntryMethod,
  suspended: bool,
  subscription: Option<SubscriptionId>,
  root_account: AccountId,
  controller_account: AccountId,
  // roles: BTreeSet<ActorInRole>
};

export class UserInfo extends Struct {
  constructor (value?: any) {
    super({
      handle: OptionText,
      avatar_uri: OptionText,
      about: OptionText
    }, value);
  }
}

export type CheckedUserInfo = {
  handle: Text,
  avatar_uri: Text,
  about: Text
};

export class PaidMembershipTerms extends Struct {
  constructor (value?: any) {
    super({
      id: PaidTermId,
      fee: u128, // BalanceOf
      text: Text
    }, value);
  }

  get id (): PaidTermId {
    return this.get('id') as PaidTermId;
  }

  get fee (): BalanceOf {
    return this.get('fee') as BalanceOf;
  }

  get text (): Text {
    return this.get('text') as Text;
  }
}

export function registerMembershipTypes () {
  try {
    const typeRegistry = getTypeRegistry();
    // Register enum EntryMethod
    typeRegistry.register({
      // Paid,
      // Screening,
      EntryMethod
    });
    typeRegistry.register({
      MemberId,
      PaidTermId,
      SubscriptionId,
      Profile: {
        handle: 'Text',
        avatar_uri: 'Text',
        about: 'Text',
        registered_at_block: 'BlockNumber',
        registered_at_time: 'Moment',
        entry: 'EntryMethod',
        suspended: 'bool',
        subscription: 'Option<SubscriptionId>',
        root_account: 'AccountId',
        controller_account: 'AccountId',
        // roles: 'BTreeSet<ActorInRole>'
      },
      UserInfo,
      CheckedUserInfo: {
        handle: 'Text',
        avatar_uri: 'Text',
        about: 'Text'
      },
      PaidMembershipTerms: {
        id: 'PaidTermId',
        fee: 'BalanceOf',
        text: 'Text'
      },
      Role,
      ActorId: 'u64',
      ActorInRole: {
        role: 'Role',
        actor_id: 'ActorId'
      },
    });
  } catch (err) {
    console.error('Failed to register custom types of membership module', err);
  }
}
