import { WsProvider } from '@polkadot/api'
import { Api, WorkingGroups } from '../Api'
import { DbService } from '../DbService'
import { config } from 'dotenv'
import Debugger from 'debug'

import creatingMemberships from '../flows/membership/creatingMemberships'
import councilSetup from '../flows/councilSetup'
import leaderSetup from '../flows/leaderSetup'
import electionParametersProposal from '../flows/proposals/electionParametersProposal'
import manageLeaderRole from '../flows/proposals/manageLeaderRole'
import spendingProposal from '../flows/proposals/spendingProposal'
import textProposal from '../flows/proposals/textProposal'
import validatorCountProposal from '../flows/proposals/validatorCountProposal'
import workingGroupMintCapacityProposal from '../flows/proposals/workingGroupMintCapacityProposal'
import atLeastValueBug from '../flows/workingGroup/atLeastValueBug'
import manageWorkerAsLead from '../flows/workingGroup/manageWorkerAsLead'
import manageWorkerAsWorker from '../flows/workingGroup/manageWorkerAsWorker'
import workerApplicaionHappyCase from '../flows/workingGroup/workerApplicationHappyCase'
import workerApplicationRejectionCase from '../flows/workingGroup/workerApplicationRejectionCase'
import workerPayout from '../flows/workingGroup/workerPayout'

const scenario = async () => {
  const debug = Debugger('scenario:full')

  // Load env variables
  config()
  const env = process.env

  // Connect api to the chain
  const nodeUrl: string = process.env.NODE_URL!
  const provider = new WsProvider(nodeUrl)
  const api: Api = await Api.create(provider)

  // Create shared state instance
  const db: DbService = DbService.getInstance()

  // Run flows serially passing them a 'context'

  debug('Memberships')
  await creatingMemberships(api, env)

  debug('Council')
  await councilSetup(api, env, db)

  // Proposals...
  debug('Election Parameters')
  await electionParametersProposal(api, env, db)

  debug('Spending Proposal')
  await spendingProposal(api, env, db)

  debug('Text Proposal')
  await textProposal(api, env, db)

  debug('Validator Count Proposal')
  await validatorCountProposal(api, env, db)

  debug('Working Group (Storage) Mint Capacity Proposal')
  await workingGroupMintCapacityProposal(api, env, db, WorkingGroups.StorageWorkingGroup)

  debug('Working Group (Content) Mint Capacity Proposal')
  await workingGroupMintCapacityProposal(api, env, db, WorkingGroups.ContentDirectoryWorkingGroup)

  // Test hiring and firing leads by the council throuh proposals
  // Leads are fired at the end of the flows
  debug('Lead Role (Storage) Proposals')
  await manageLeaderRole(api, env, db, WorkingGroups.StorageWorkingGroup)
  debug('Lead Role (Content) Proposals')
  await manageLeaderRole(api, env, db, WorkingGroups.ContentDirectoryWorkingGroup)

  /* workers tests */

  debug('Sudo Hiring Lead (Storage)')
  await leaderSetup(api, env, db, WorkingGroups.StorageWorkingGroup)
  await atLeastValueBug(api, env, db)

  debug('Storage Worker Tests')
  await manageWorkerAsLead(api, env, db, WorkingGroups.StorageWorkingGroup)
  await manageWorkerAsWorker(api, env, db, WorkingGroups.StorageWorkingGroup)
  await workerApplicaionHappyCase(api, env, db, WorkingGroups.StorageWorkingGroup)
  await workerApplicationRejectionCase(api, env, db, WorkingGroups.StorageWorkingGroup)
  await workerPayout(api, env, db, WorkingGroups.StorageWorkingGroup)

  debug('Sudo Hiring Lead (Content)')
  await leaderSetup(api, env, db, WorkingGroups.ContentDirectoryWorkingGroup)

  debug('Content Worker Tests')
  await manageWorkerAsLead(api, env, db, WorkingGroups.ContentDirectoryWorkingGroup)
  await manageWorkerAsWorker(api, env, db, WorkingGroups.ContentDirectoryWorkingGroup)
  await workerApplicaionHappyCase(api, env, db, WorkingGroups.ContentDirectoryWorkingGroup)
  await workerApplicationRejectionCase(api, env, db, WorkingGroups.ContentDirectoryWorkingGroup)
  await workerPayout(api, env, db, WorkingGroups.ContentDirectoryWorkingGroup)

  // Note: disconnecting and then reconnecting to the chain in the same process
  // doesn't seem to work!
  api.close()
}

scenario()
