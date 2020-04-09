import AccountsCommandBase from '../../base/AccountsCommandBase';
import Api from '../../Api';
import { AccountSummary, NameValueObj, NamedKeyringPair } from '../../Types';
import { displayHeader, displayNameValueTable } from '../../helpers/display';
import { formatBalance } from '@polkadot/util';
import moment from 'moment';

export default class AccountCurrent extends AccountsCommandBase {
    static description = 'Display information about currently choosen default account';
    static aliases = ['account:info', 'account:default'];

    async run() {
        const api: Api = new Api();
        const currentAccount: NamedKeyringPair = await this.getRequiredSelectedAccount(false);
        const summary: AccountSummary = await api.getAccountSummary(currentAccount.address);

        displayHeader('Account information');
        const creationDate: string = currentAccount.meta.whenCreated ?
            moment(currentAccount.meta.whenCreated).format('YYYY-MM-DD HH:mm:ss')
            : '?';
        const accountRows: NameValueObj[] = [
            { name: 'Account name:', value: currentAccount.meta.name },
            { name: 'Address:', value: currentAccount.address },
            { name: 'Created:', value: creationDate }
        ];
        displayNameValueTable(accountRows);

        displayHeader('Balances');
        const balancesRows: NameValueObj[] = [
            { name: 'Available balance:', value: formatBalance(summary.balances.availableBalance) },
            { name: 'Free balance:', value: formatBalance(summary.balances.freeBalance) },
            { name: 'Locked balance:', value: formatBalance(summary.balances.lockedBalance) }
        ];
        displayNameValueTable(balancesRows);
    }
  }
