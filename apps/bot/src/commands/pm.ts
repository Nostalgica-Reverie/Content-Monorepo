import process from 'node:process'

import {
	ApplicationCommandType,
	ChatInputCommandInteraction,
	GuildMemberRoleManager,
	PermissionFlagsBits,
	SlashCommandBuilder,
} from 'discord.js'

import { PERMISSION_ERROR_TEXT } from '@/data'
import { info } from '@/logging/logger'
import { ChatInputCommand } from '@/types'
import { createDefaultEmbed } from '@/utils'

export const pmCommand: ChatInputCommand = {
	type: ApplicationCommandType.ChatInput,
	data: new SlashCommandBuilder()
		.setName('pm')
		.setDescription('Send a private message')
		.setDefaultMemberPermissions(PermissionFlagsBits.BanMembers)
		.addUserOption((option) =>
			option.setName('user').setDescription('Discord User').setRequired(true),
		)
		.addStringOption((option) =>
			option.setName('message').setDescription('Private message').setRequired(true),
		)
		.addStringOption((option) =>
			option.setName('title').setDescription('Embed title').setRequired(false),
		) as SlashCommandBuilder,
	meta: {
		name: 'pm',
		description: 'Send a private message',
		category: 'moderation',
		guildOnly: true,
	},
	execute: async (interaction: ChatInputCommandInteraction) => {
		if (!interaction.guild || !interaction.member) return
		if (
			!(interaction.member.roles as GuildMemberRoleManager).cache.has(
				process.env.DISCORD_MODERATOR_ROLE_ID!,
			)
		) {
			await interaction.reply({
				content: PERMISSION_ERROR_TEXT,
				flags: 'Ephemeral',
			})
			return
		}

		const user = interaction.options.getUser('user', true)
		const message = interaction.options.getString('message', true)
		const title = interaction.options.getString('title', false)
		const member = await interaction.guild.members.fetch(user.id)

		const embed = createDefaultEmbed({
			title: title ? title : 'Message from a moderator',
			description: message,
		})

		const previewMessage = `SUBJECT: ${title ? title : 'Message from a moderator'}\n\n${message}`

		await member.user.send({ embeds: [embed] })
		await interaction.reply({
			content: 'Private message has been successfully sent!',
			flags: 'Ephemeral',
		})
		info(
			`:incoming_envelope: Moderator (\`${interaction.user.username}\`, ID: ${interaction.user.id}) has sent a private message to a user ${member.user} (\`${member.user.username}\`, ID: ${member.user.id}):\n\`\`\`${previewMessage}\`\`\``,
		)
	},
}
