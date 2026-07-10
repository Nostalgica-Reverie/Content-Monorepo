import { ApplicationCommandType, SlashCommandBuilder } from 'discord.js'

import type { ChatInputCommand } from '@/types/commands'

export const githubCommand: ChatInputCommand = {
	type: ApplicationCommandType.ChatInput,
	data: new SlashCommandBuilder()
		.setName('github')
		.setDescription("Query Reverie's GitHub repositories")
		.addStringOption((option) =>
			option
				.setName('path')
				.setDescription('Repository path (e.g., "code/issues", "launcher", "docs")')
				.setRequired(true),
		) as SlashCommandBuilder,
	meta: {
		name: 'github',
		description: "Query Reverie's GitHub repositories",
		category: 'utility',
		cooldownSeconds: 3,
	},
	async execute(interaction) {
		const path = interaction.options.getString('path', true)

		const cleanPath = path.trim().replace(/\s+/g, '')

		const githubUrl = `https://github.com/Nostalgica-Reverie/${cleanPath}`

		await interaction.reply({
			content: githubUrl,
		})
	},
}
