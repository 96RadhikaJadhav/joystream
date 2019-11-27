import { MusicAlbumSample } from "./MusicAlbumSamples";
import { TracksOfMyMusicAlbumProps } from "@polkadot/joy-media/music/MusicAlbumTracks";
import { MusicAlbumEntity } from "@polkadot/joy-media/entities/MusicAlbumEntity";

export const trackNames = [
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

export const albumTracks = trackNames.map((title, i) => ({
	id: `${i}`,
	title,
	artist: 'Man from the Woods',
	cover: 'https://images.unsplash.com/photo-1477414348463-c0eb7f1359b6?ixlib=rb-1.2.1&auto=format&fit=crop&w=200&q=60'
}));

export const MusicTrackSamples = 
	trackNames.slice(0, trackNames.length / 2).map((title, i) => ({
		id: `${100 + i}`,
		title,
		artist: 'Man from the Woods',
		cover: 'https://images.unsplash.com/photo-1477414348463-c0eb7f1359b6?ixlib=rb-1.2.1&auto=format&fit=crop&w=200&q=60'
	}))
	.concat(
		trackNames.slice(trackNames.length / 2).map((title, i) => ({
			id: `${200 + i}`,
			title,
			artist: 'Liquid Stone',
			cover: 'https://images.unsplash.com/photo-1484352491158-830ef5692bb3?ixlib=rb-1.2.1&auto=format&fit=crop&w=500&q=60',
		}))
	)

  export const AlbumWithTracksProps: TracksOfMyMusicAlbumProps = {
	album: MusicAlbumSample,
	tracks: albumTracks
}

export const MusicAlbumExample: MusicAlbumEntity = {
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
