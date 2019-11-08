import React from 'react';
import './index.css';

import { withKnobs } from '@storybook/addon-knobs';
import { MyMusicAlbums } from './MyMusicAlbums';
import { AlbumExample } from './StorybookUtils';

export default { 
    title: 'Media | My music albums',
    decorators: [withKnobs],
};

export const DefaultState = () => {
	return <MyMusicAlbums />;
}

export const FewMusicAlbums = () => {
	return <MyMusicAlbums albums={fewMusicAlbums} />;
}

const fewMusicAlbums = [
	AlbumExample,
	{
		title: 'Riddle',
		artist: 'Liquid Stone',
		cover: 'https://images.unsplash.com/photo-1484352491158-830ef5692bb3?ixlib=rb-1.2.1&auto=format&fit=crop&w=500&q=60',
		tracksCount: 1
	},
	{
		title: 'Habitants of the silver water',
		artist: 'Heavy Waves and Light Shells',
		cover: 'https://images.unsplash.com/photo-1543467091-5f0406620f8b?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=crop&w=200&q=60',
		tracksCount: 12
	}
]