import { newEntityId } from './EntityId.mock';
import { PublicationStatusType } from '../schemas/general/PublicationStatus';

const values = [
  ''
];

export const AllPublicationStatuses: PublicationStatusType[] =
  values.map(value => ({ id: newEntityId(), value }));

export const PublicationStatus = AllPublicationStatuses[0];
