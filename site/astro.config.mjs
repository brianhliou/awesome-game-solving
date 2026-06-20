// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// One sidebar group per genre, generated from data/games/*.yaml by
// scripts/gen-content.ts (runs on predev/prebuild).
import gamesSidebar from './src/generated/sidebar.json' with { type: 'json' };

// https://astro.build/config
export default defineConfig({
	site: 'https://solved.brianhliou.com',
	integrations: [
		starlight({
			title: 'Solved Games',
			description:
				'The reference for which games are solved — by whom, how strongly, and what is left. Backed by an open-source solver that ships first-published solutions.',
			social: [
				{ icon: 'github', label: 'GitHub', href: 'https://github.com/brianhliou/solved-games' },
			],
			sidebar: [
				{
					label: 'Overview',
					items: [
						{ label: 'Home', link: '/' },
						{ label: 'Open frontier', link: '/frontier/' },
					],
				},
				{
					label: 'Families',
					items: [
						{ label: 'Y (connection)', link: '/families/y/' },
						{ label: 'Morris (mill games)', link: '/families/morris/' },
					],
				},
				...gamesSidebar,
			],
		}),
	],
});
