// Copyright 2017-2019 @polkadot/app-staking authors & contributors
// This software may be modified and distributed under the terms
// of the Apache-2.0 license. See the LICENSE file for details.

import { BareProps } from './types';

import BN from 'bn.js';
import React from 'react';
import { AccountId, AccountIndex, Address, Balance } from '@polkadot/types/interfaces';
import { withCall, withMulti } from '@polkadot/react-api/index';

import classes from './util/classes';
import toShortAddress from './util/toShortAddress';
import BalanceDisplay from './Balance';
import IdentityIcon from './IdentityIcon';
import { findNameByAddress, nonEmptyStr } from '@polkadot/joy-utils/index';
import MemoView from '@polkadot/joy-utils/memo/MemoView';

type Props = BareProps & {
  balance?: Balance | Array<Balance> | BN,
  children?: React.ReactNode,
  isPadded?: boolean,
  isShort?: boolean,
  session_validators?: Array<AccountId>,
  value?: AccountId | AccountIndex | Address | string,
  name?: string,
  size?: number,
  withAddress?: boolean,
  withBalance?: boolean,
  withName?: boolean,
  withMemo?: boolean
};

class AddressMini extends React.PureComponent<Props> {
  render () {
    const { children, className, isPadded = true, session_validators, style, size, value } = this.props;

    if (!value) {
      return null;
    }

    const address = value.toString();
    const isValidator = (session_validators || []).find((validator) =>
      validator.toString() === address
    );

    return (
      <div
        className={classes('ui--AddressMini', isPadded ? 'padded' : '', className)}
        style={style}
      >
        <div className='ui--AddressMini-info'>
          <IdentityIcon
            isHighlight={!!isValidator}
            size={size || 36}
            value={address}
          />
          <div>
            {this.renderAddress(address)}
            <div className='ui--AddressMini-details'>
              {this.renderName(address)}
              {this.renderBalance()}
              {this.renderMemo(address)}
            </div>
          </div>
          {children}
        </div>
      </div>
    );
  }

  private renderAddress (address: string) {
    const { isShort = true, withAddress = true } = this.props;
    if (!withAddress) {
      return null;
    }

    return (
      <div className='ui--AddressMini-address'>
        {isShort ? toShortAddress(address) : address}
      </div>
    );
  }

  private renderName (address: string) {
    let { name, withName = false } = this.props;
    if (!withName) {
      return null;
    }

    name = name ? name : findNameByAddress(address);
    return (nonEmptyStr(name) ?
      <div className='ui--AddressSummary-name'>
        Name: <b style={{ textTransform: 'uppercase' }}>{name}</b>
      </div> : null
    );
  }

  private renderBalance () {
    const { balance, value, withBalance = false } = this.props;
    if (!withBalance || !value) {
      return null;
    }

    return (
      <BalanceDisplay
        label='Balance: '
        balance={balance}
        className='ui--AddressSummary-balance'
        params={value}
      />
    );
  }

  private renderMemo (address: string) {
    let { withMemo = false } = this.props;
    if (!withMemo) {
      return null;
    }

    return <div className='ui--AddressSummary-memo'>
      Memo: <b><MemoView accountId={address} preview={true} showEmpty={true} /></b>
    </div>;
  }
}

export default withMulti(
  AddressMini,
  withCall('query.session.validators')
);

type AddressPreviewProps = {
  address: AccountId | AccountIndex | Address | string
};

export function AddressPreview ({ address }: AddressPreviewProps) {
  return <AddressMini
    value={address}
    isShort={false}
    isPadded={false}
    withBalance={true}
    withName={true}
    withMemo={false}
    size={36}
  />;
}
