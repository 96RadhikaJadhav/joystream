import { MusicAlbumPreviewProps } from "./MyMusicAlbums";

export const AlbumPreviewExample: MusicAlbumPreviewProps = {
  title: 'Sound of the cold leaves',
  artist: 'Man from the Woods',
  cover: 'https://images.unsplash.com/photo-1477414348463-c0eb7f1359b6?ixlib=rb-1.2.1&auto=format&fit=crop&w=200&q=60',
  tracksCount: 8
};

export const FewMusicAlbums = [
	AlbumPreviewExample,
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