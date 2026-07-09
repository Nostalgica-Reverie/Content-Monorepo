import {
	ApplicationCommandType,
	type ChatInputCommandInteraction,
	Collection,
	Interaction,
	type MessageContextMenuCommandInteraction,
	PermissionsBitField,
	REST,
	Routes,
} from 'discord.js'

import commands from '../commands/'
import type {
	AnyCommand,
	ChatInputCommand,
	CommandHandlerOptions,
	CommandHandlers,
	CommandMap,
	MessageContextMenuCommand,
} from '../types/commands'

type CommandType = ApplicationCommandType.ChatInput | ApplicationCommandType.Message
type CooldownKey = string // `${userId}:${commandType}:${commandName}`
type CommandInteraction = ChatInputCommandInteraction | MessageContextMenuCommandInteraction

function getCommandType(cmd: AnyCommand): CommandType {
	return cmd.type
}

function getCommandKey(type: CommandType, name: string) {
	return `${type}:${name}`
}

function isChatInputCommand(cmd: AnyCommand): cmd is ChatInputCommand {
	return getCommandType(cmd) === ApplicationCommandType.ChatInput
}

function isMessageContextMenuCommand(cmd: AnyCommand): cmd is MessageContextMenuCommand {
	return getCommandType(cmd) === ApplicationCommandType.Message
}

export function createCommandRegistry(
	commands: AnyCommand[],
	opts: CommandHandlerOptions = {},
): CommandHandlers {
	const map: CommandMap = new Map(
		commands.map((c) => [getCommandKey(getCommandType(c), c.meta.name), c]),
	)
	const cooldowns = new Collection<CooldownKey, number>()
	const defaultCooldown = opts.defaultCooldownSeconds ?? 3

	function isAllowedGuild(cmd: AnyCommand, guildId?: string | null) {
		if (!guildId) return !(cmd.meta.guildOnly ?? false)
		if (cmd.meta.allowedGuilds?.length) {
			return cmd.meta.allowedGuilds.includes(guildId)
		}
		return true
	}

	function isAllowedUser(cmd: AnyCommand, userId: string) {
		if (cmd.meta.allowedUsers?.length) {
			return cmd.meta.allowedUsers.includes(userId)
		}
		return true
	}

	function checkCooldown(userId: string, cmd: AnyCommand, seconds: number) {
		const key = `${userId}:${getCommandType(cmd)}:${cmd.meta.name}`
		const now = Date.now()
		const until = cooldowns.get(key)
		if (until && until > now) {
			return Math.ceil((until - now) / 1000)
		}
		cooldowns.set(key, now + seconds * 1000)
		return 0
	}

	async function runCommand(
		interaction: CommandInteraction,
		cmd: AnyCommand,
		execute: () => Promise<void> | void,
	) {
		// Context checks
		if (cmd.meta.dmOnly && interaction.inGuild()) return
		if (cmd.meta.guildOnly && !interaction.inGuild()) return
		if (!isAllowedGuild(cmd, interaction.guildId)) return
		if (!isAllowedUser(cmd, interaction.user.id)) return

		// Cooldown
		const cd = cmd.meta.cooldownSeconds ?? defaultCooldown
		if (cd > 0) {
			const remain = checkCooldown(interaction.user.id, cmd, cd)
			if (remain > 0) {
				if (interaction.deferred || interaction.replied) return
				const commandName =
					getCommandType(cmd) === ApplicationCommandType.ChatInput
						? `/${cmd.meta.name}`
						: `"${cmd.meta.name}"`
				await interaction.reply({
					content: `Please wait ${remain}s before using ${commandName} again.`,
					flags: 'Ephemeral',
				})
				return
			}
		}

		try {
			await Promise.resolve(execute())
		} catch (err) {
			if (opts.debug) console.error(`[command:${cmd.meta.name}]`, err)
			const content = 'There was an error while executing this command.'
			if (interaction.deferred || interaction.replied) {
				await interaction.followUp({ content, flags: 'Ephemeral' }).catch(() => {})
			} else {
				await interaction.reply({ content, flags: 'Ephemeral' }).catch(() => {})
			}
		}
	}

	async function onInteractionCreate(interaction: Interaction) {
		if (interaction.isChatInputCommand()) {
			const cmd = map.get(getCommandKey(ApplicationCommandType.ChatInput, interaction.commandName))
			if (!cmd || !isChatInputCommand(cmd)) return
			await runCommand(interaction, cmd, () => cmd.execute(interaction))
			return
		}

		if (interaction.isMessageContextMenuCommand()) {
			const cmd = map.get(getCommandKey(ApplicationCommandType.Message, interaction.commandName))
			if (!cmd || !isMessageContextMenuCommand(cmd)) return
			await runCommand(interaction, cmd, () => cmd.execute(interaction))
		}
	}

	function getAllApplicationCommandData() {
		return Array.from(map.values()).map((c) => {
			if (c.meta.defaultMemberPermissions !== undefined) {
				const bits = new PermissionsBitField(c.meta.defaultMemberPermissions as any).bitfield
				c.data.setDefaultMemberPermissions(bits)
			}
			if (c.meta.dmPermission !== undefined) {
				c.data.setDMPermission(c.meta.dmPermission)
			}
			return c.data.toJSON()
		})
	}

	return {
		onInteractionCreate,
		getAllApplicationCommandData,
	}
}

export async function deployCommands() {
	const token = process.env.DISCORD_BOT_TOKEN
	const clientId = process.env.DISCORD_CLIENT_ID
	const guildId = process.env.GUILD_ID

	if (!token || !clientId || !guildId) {
		console.error('Missing DISCORD_BOT_TOKEN, DISCORD_CLIENT_ID or GUILD_ID')
		process.exit(1)
	}

	const registry = createCommandRegistry(commands)
	const body = registry.getAllApplicationCommandData()

	const rest = new REST().setToken(token)

	try {
		await rest.put(Routes.applicationCommands(clientId), { body: [] })
		console.log('Successfully deleted all application commands.')

		await rest
			.put(Routes.applicationGuildCommands(clientId, guildId), { body: [] })
			.then(() => console.log('Successfully deleted all guild commands.'))
			.catch(console.error)

		console.log(body)
		await rest.put(Routes.applicationCommands(clientId), { body })
		console.log(`Registered ${body.length} application command(s).`)
	} catch (err) {
		console.error(err)
	}
}
