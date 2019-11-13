import React from 'react';
import '../music/index.css';

import { withKnobs } from '@storybook/addon-knobs';
import { TracksOfMyMusicAlbum, TracksOfMyMusicAlbumProps } from '../music/MusicAlbumTracks';
import { AlbumPreviewExample } from '../music/StorybookUtils';
import { ReorderTracksInAlbum } from '../music/ReorderTracksInAlbum';
import { EditAlbumModal } from '../music/EditAlbumModal';
import { MusicAlbumEntity } from '../entities/MusicAlbumEntity';
import { EditMusicAlbum } from '../music/EditMusicAlbum';

export default { 
    title: 'Media | Tracks of my music album',
    decorators: [withKnobs],
};

export const DefaultState = () => {
	return <TracksOfMyMusicAlbum album={AlbumPreviewExample} />;
}

export const AlbumWithTracks = () => {
	return <TracksOfMyMusicAlbum {...AlbumWithTracksProps} />
}

export const ReorderTracks = () =>
	<ReorderTracksInAlbum {...AlbumWithTracksProps} />

export const EditAlbumModalStory = () =>
	<EditAlbumModal {...AlbumWithTracksProps} />

export const EditAlbumStory = () =>
	<EditMusicAlbum
		isStorybook={true} 
		entity={MusicAlbumExample}
		tracks={albumTracks}
	/>

const trackNames = [
	'Arborvitae (Thuja occidentalis)',
	'Black Ash (Fraxinus nigra)',
	'White Ash (Fraxinus americana)',
	'Bigtooth Aspen (Populus grandidentata)',
	'Quaking Aspen (Populus tremuloides)',
	'Basswood (Tilia americana)',
	'American Beech (Fagus grandifolia)',
	'Black Birch (Betula lenta)',
	'Gray Birch (Betula populifolia)',
	'Paper Birch (Betula papyrifera)',
	'Yellow Birch (Betula alleghaniensis)',
	'Butternut (Juglans cinerea)',
	'Black Cherry (Prunus serotina)',
	'Pin Cherry (Prunus pensylvanica)'
]

const albumTracks = trackNames.map(title => ({
	title,
	artist: 'Man from the Woods',
	cover: 'https://images.unsplash.com/photo-1477414348463-c0eb7f1359b6?ixlib=rb-1.2.1&auto=format&fit=crop&w=200&q=60'
}));

const AlbumWithTracksProps: TracksOfMyMusicAlbumProps = {
	album: AlbumPreviewExample,
	tracks: albumTracks
}

const MusicAlbumExample: MusicAlbumEntity = {
	title: 'Requiem (Mozart)',
	about: 'The Requiem in D minor, K. 626, is a requiem mass by Wolfgang Amadeus Mozart (1756–1791). Mozart composed part of the Requiem in Vienna in late 1791, but it was unfinished at his death on 5 December the same year. A completed version dated 1792 by Franz Xaver Süssmayr was delivered to Count Franz von Walsegg, who commissioned the piece for a Requiem service to commemorate the anniversary of his wifes death on 14 February.',
	cover: 'https://assets.classicfm.com/2017/36/mozart-1504532179-list-handheld-0.jpg',
	year: 2019,

	// visibility: 'Public',
	// album: 'Greatest Collection of Mozart',

	// Additional:
	artist: 'Berlin Philharmonic',
	composer: 'Wolfgang Amadeus Mozart',
	genre: 'Classical Music',
	mood: 'Relaxing',
	theme: 'Dark',
	explicit: false,
	license: 'Public Domain',
};