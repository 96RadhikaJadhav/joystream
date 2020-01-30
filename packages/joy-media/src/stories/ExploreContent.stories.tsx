import React from 'react';
import '../common/index.css';

import { ExploreContent } from '../explore/ExploreContent';
import { MusicAlbumSamples, FeaturedAlbums } from './data/MusicAlbumSamples';
import { PlayContent } from '../explore/PlayContent';
import { Album1TrackSamples } from './data/MusicTrackSamples';
import { ChannelDataSample } from './data/ChannelSamples';
import { withMockTransport } from './withMockTransport';

export default { 
	title: 'Media | Explore',
	decorators: [ withMockTransport ],
};

export const DefaultState = () =>
	<ExploreContent />;

export const FeaturedAndLatestAlbums = () =>
	<ExploreContent 
		featuredAlbums={FeaturedAlbums}
		latestAlbums={MusicAlbumSamples.reverse()}
	/>;

export const PlayAlbum = () =>
	<PlayContent 
		channel={ChannelDataSample}
		featuredAlbums={FeaturedAlbums}
		tracks={Album1TrackSamples}
		currentTrackIndex={3}
	/>;
