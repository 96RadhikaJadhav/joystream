import React from 'react';
import { MusicAlbumPreviewProps, MusicAlbumPreview } from '../music/MusicAlbumPreview';
import { MusicTrackReaderPreviewProps, MusicTrackReaderPreview } from '../music/MusicTrackReaderPreview';
import { Pluralize } from '@polkadot/joy-utils/Pluralize';
import { Table } from 'semantic-ui-react';

type Props = {
  tracks: MusicTrackReaderPreviewProps[],
  currentTrackIndex?: number,
  featuredAlbums?: MusicAlbumPreviewProps[],
};

// TODO get meta from track item
const meta = {
  artist: 'Berlin Philharmonic',
	composer: 'Wolfgang Amadeus Mozart',
	genre: 'Classical Music',
	mood: 'Relaxing',
	theme: 'Dark',
	explicit: false,
	license: 'Public Domain'
}

export function PlayContent (props: Props) {
  const { tracks = [], currentTrackIndex = 0, featuredAlbums = [] } = props;

  const currentTrack = tracks[currentTrackIndex];

  const metaField = (label: React.ReactNode, value: React.ReactNode) =>
    <Table.Row>
      <Table.Cell width={2}>{label}</Table.Cell>
      <Table.Cell>{value}</Table.Cell>
    </Table.Row>

  const metaTable = <>
    <h3>Track Info</h3>
    <Table basic='very' compact className='JoyPlayAlbum_MetaInfo'>
      <Table.Body>
        {metaField('Artist', meta.artist)}
        {metaField('Composer', meta.composer)}
        {metaField('Genre', meta.genre)}
        {metaField('Mood', meta.mood)}
        {metaField('Theme', meta.theme)}
        {metaField('Explicit', meta.explicit ? 'Yes' : 'No')}
        {metaField('License', meta.license)}
      </Table.Body>
    </Table>
  </>

  return <div className='JoyPlayAlbum'>
    <div className='JoyPlayAlbum_Main'>
      <div className='JoyPlayAlbum_CurrentTrack'>
        <MusicTrackReaderPreview {...currentTrack} size={400} />
        {metaTable}
      </div>
      <div className='JoyPlayAlbum_AlbumTracks'>
        <h3><Pluralize count={tracks.length} singularText='Track' /></h3>
        {tracks.map((x, i) => 
          <div className='Track'>
            <span className='TrackNumber'>{i + 1}</span>
            <MusicTrackReaderPreview {...x} orientation='horizontal' size={60} />
          </div>
        )}
      </div>
    </div>
    {featuredAlbums.length > 0 &&
      <div className='JoyPlayAlbum_Featured'>
        <h3>Featured albums</h3>
        {featuredAlbums.map(x => <MusicAlbumPreview {...x} size={170} />)}
      </div>
    }
  </div>;
}