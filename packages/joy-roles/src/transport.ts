import { Balance } from '@polkadot/types/interfaces';

import { Role } from '@joystream/types/roles';

import { WorkingGroupProps, StorageAndDistributionProps } from "./tabs/WorkingGroup"
import { 
  WorkingGroupOpening,
} from "./tabs/Opportunities"

export interface ITransport {
  roles: () => Promise<Array<Role>>
  curationGroup: () => Promise<WorkingGroupProps>
  storageGroup: () => Promise<StorageAndDistributionProps>
  currentOpportunities: () => Promise<Array<WorkingGroupOpening>>
  opening: (id: string) => Promise<WorkingGroupOpening>
  openingApplicationRanks: (openingId: string) => Promise<Balance[]>
  expectedBlockTime: () => Promise<number>
  transactionFee: () => Promise<Balance>
}
