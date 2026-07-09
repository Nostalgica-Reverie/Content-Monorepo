import type {
	AutocompleteInteraction,
	ChatInputCommandInteraction,
	ContextMenuCommandBuilder,
	MessageContextMenuCommandInteraction,
	PermissionResolvable,
	SlashCommandBuilder,
} from 'discord.js'
import { ApplicationCommandType } from 'discord.js'

export type CommandCategory = 'general' | 'moderation' | 'utility' | 'fun' | 'admin'

export interface CommandMeta {
	name: string
	description: string
	category?: CommandCategory
	// If true, only allow in guilds
	guildOnly?: boolean
	// If true, only allow in DMs
	dmOnly?: boolean
	// If set, restrict command to these guild IDs (e.g., for dev commands)
	allowedGuilds?: string[]
	// If set, restrict command to these user IDs
	allowedUsers?: string[]
	// Per-user cooldown in seconds
	cooldownSeconds?: number
	// Required member permissions in a guild to execute
	defaultMemberPermissions?: PermissionResolvable
	// Enables Discord-level DM permission (defaults true)
	dmPermission?: boolean
}

type CommandDataBuilder = {
	toJSON(): any
	setDefaultMemberPermissions(permissions: any): any
	setDMPermission(enabled: boolean): any
}

interface BaseCommand<
	TInteraction extends ChatInputCommandInteraction | MessageContextMenuCommandInteraction,
	TData extends CommandDataBuilder,
	TType extends ApplicationCommandType.ChatInput | ApplicationCommandType.Message,
> {
	type: TType
	data: TData
	meta: CommandMeta
	execute: (interaction: TInteraction) => Promise<void> | void
}

export type ChatInputCommand = BaseCommand<
	ChatInputCommandInteraction,
	SlashCommandBuilder & CommandDataBuilder,
	ApplicationCommandType.ChatInput
> & {
	autocomplete?: (interaction: AutocompleteInteraction) => Promise<void> | void
}

export type MessageContextMenuCommand = BaseCommand<
	MessageContextMenuCommandInteraction,
	ContextMenuCommandBuilder & CommandDataBuilder,
	ApplicationCommandType.Message
>

export type AnyCommand = ChatInputCommand | MessageContextMenuCommand

export type CommandMap = Map<string, AnyCommand>

export interface CommandHandlerOptions {
	owners?: string[]
	// Default cooldown if a command doesn't specify one
	defaultCooldownSeconds?: number
	// Log execution and errors
	debug?: boolean
}

export interface CommandHandlers {
	onInteractionCreate: (interaction: any) => Promise<void>
	getAllApplicationCommandData: () => any[]
}
