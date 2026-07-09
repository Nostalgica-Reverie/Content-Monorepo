import { ApplicationCommandType, SlashCommandBuilder } from 'discord.js'

import type { ChatInputCommand } from '@/types/commands'
import { createDefaultEmbed } from '@/utils'
import { createVerificationState } from '@/web'

export const verifyCommand: ChatInputCommand = {
	type: ApplicationCommandType.ChatInput,
	data: new SlashCommandBuilder()
		.setName('verify')
		.setDescription('Link your external account to your Discord user')
		.addSubcommand((sub) =>
			sub.setName('crowdin').setDescription('Link your Crowdin account'),
		) as SlashCommandBuilder,
	meta: {
		name: 'verify',
		description: 'Link your Crowdin account with your Discord user',
		category: 'utility',
		guildOnly: true,
	},
	execute: async (interaction) => {
		const sub = interaction.options.getSubcommand()
		if (sub === 'crowdin') {
			const base = process.env.PUBLIC_BASE_URL || 'http://localhost:3000'
			const token = await createVerificationState(interaction.user.id)
			const url = `${base}/crowdin/verify?token=${encodeURIComponent(token)}`

			const expireAt = Math.floor(Date.now() / 1000) + 15 * 60 // now + 15 minutes

			const embed = createDefaultEmbed()
				.setTitle('Link your Crowdin account')
				.setDescription(
					[
						'We need to verify your Crowdin account to link it with your Discord.',
						' ',
						'To continue, please click the link down below.',
						' ',
						`**[[ Click here to continue → ]](${url})**`,
						' ',
						`-# This link will expire <t:${expireAt}:R>`,
					].join('\n'),
				)

			await interaction.reply({
				embeds: [embed],
				flags: 'Ephemeral',
			})
			return
		}
	},
}

export default verifyCommand
