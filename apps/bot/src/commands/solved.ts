import { ApplicationCommandType, SlashCommandBuilder, Snowflake } from 'discord.js'

import { FORUM_ONLY_ERROR_TEXT, OP_MARKED_SOLVED_TEXT, OP_ONLY_ERROR_TEXT } from '@/data'
import type { ChatInputCommand } from '@/types/commands'

export const solvedCommand: ChatInputCommand = {
	type: ApplicationCommandType.ChatInput,
	data: new SlashCommandBuilder().setName('solved').setDescription('Mark your thread as solved'),
	meta: {
		name: 'solved',
		description: 'Mark your thread as solved',
		category: 'utility',
	},
	async execute(interaction) {
		if (interaction.channel?.isThread()) {
			if (interaction.channel.ownerId === interaction.user.id) {
				await interaction.reply(OP_MARKED_SOLVED_TEXT)

				await interaction.channel.edit({
					archived: true,
					appliedTags: [
						...interaction.channel.appliedTags,
						process.env.COMMUNITY_SUPPORT_FORUM_SOLVED_TAG_ID as Snowflake,
					],
				})
			} else {
				await interaction.reply({
					content: OP_ONLY_ERROR_TEXT,
					flags: 'Ephemeral',
				})
			}
		} else {
			await interaction.reply({
				content: FORUM_ONLY_ERROR_TEXT,
				flags: 'Ephemeral',
			})
		}
	},
}
