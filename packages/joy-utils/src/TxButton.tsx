import { BareProps, ApiProps } from '@polkadot/react-api/types';
import { QueueTxExtrinsicAdd, PartialQueueTxExtrinsic } from '@polkadot/react-components/Status/types';

import React from 'react';
import { Button } from '@polkadot/react-components/index';
import { QueueConsumer } from '@polkadot/react-components/Status/Context';
import { withApi } from '@polkadot/react-api/index';
import { assert } from '@polkadot/util';
import { withMyAccount, MyAccountProps } from '@polkadot/joy-utils/MyAccount';

type InjectedProps = {
  queueExtrinsic: QueueTxExtrinsicAdd;
};

type Props = BareProps & ApiProps & MyAccountProps & PartialQueueTxExtrinsic & {
  accountId?: string,
  type?: 'submit' | 'button',
  isPrimary?: boolean,
  isDisabled?: boolean,
  label: React.ReactNode,
  params: Array<any>,
  tx: string,
  onClick?: (sendTx: () => void) => void
};

class TxButtonInner extends React.PureComponent<Props & InjectedProps> {
  render () {
    const { myAddress, accountId, isPrimary = true, isDisabled, label, onClick } = this.props;
    const origin = accountId || myAddress;

    return (
      <Button
        {...this.props}
        isDisabled={isDisabled || !origin}
        isPrimary={isPrimary}
        label={label}
        onClick={() => {
          if (onClick) onClick(this.send);
          else this.send();
        }}
      />
    );
  }

  private send = (): void => {
    const {
      myAddress, accountId, api, params, queueExtrinsic, tx,
      txFailedCb, txSuccessCb, txStartCb, txUpdateCb,
    } = this.props;
    const origin = accountId || myAddress;
    const [section, method] = tx.split('.');

    assert(api.tx[section] && api.tx[section][method], `Unable to find api.tx.${section}.${method}`);

    queueExtrinsic({
      accountId: origin,
      extrinsic: api.tx[section][method](...params) as any, // ???
      txFailedCb,
      txSuccessCb,
      txStartCb,
      txUpdateCb,
    });
  }
}

class TxButton extends React.PureComponent<Props> {
  render () {
    return (
      <QueueConsumer>
        {({ queueExtrinsic }) => (
          <TxButtonInner
            {...this.props}
            queueExtrinsic={queueExtrinsic}
          />
        )}
      </QueueConsumer>
    );
  }
}

export default withApi(withMyAccount(TxButton));
