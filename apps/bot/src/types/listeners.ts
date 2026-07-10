import {
	Client,
	Message,
	MessageReaction,
	PartialMessage,
	PartialMessageReaction,
	PartialUser,
	PermissionResolvable,
	User,
} from 'discord.js'

/**
 * Immutable context for handling a Discord message within the listener pipeline.
 * - Captures the Discord client, the message, and a timestamp for consistent computations.
 * - Optionally carries application services for dependency injection (e.g., db, cache, config).
 * @public
 */
export type MessageContext = {
	/** Discord.js client instance. */
	client: Client
	/** The incoming message to evaluate. */
	message: Message
	/** Milliseconds since the UNIX epoch captured at pipeline start. */
	now: number
	/** Optional dependency container for app services (db, cache, config). */
	services?: Record<string, unknown>
}

/** Context for message edit events. @public */
export type MessageUpdateContext = {
	client: Client
	oldMessage: Message | PartialMessage
	newMessage: Message | PartialMessage
	now: number
	services?: Record<string, unknown>
}

/** Context for message delete events. @public */
export type MessageDeleteContext = {
	client: Client
	message: Message | PartialMessage
	now: number
	services?: Record<string, unknown>
}

/**
 * Cheap, synchronous checks evaluated before the match predicate.
 * Defaults: bots are rejected and DMs are rejected unless allowDMs is true.
 * All provided constraints are AND-ed together.
 * Notes: Role and permission checks are only applied in guild contexts (messages with a member).
 * @public
 */
export type ListenerFilter = {
	/** Allow messages from bot accounts. Defaults to false. */
	allowBots?: boolean
	/** Allow direct messages. Defaults to false. */
	allowDMs?: boolean
	/** Only allow messages from these guilds. */
	guildIds?: string[]
	/** Only allow messages from these channels. */
	channelIds?: string[]
	/** Only allow authors with one of these user IDs. */
	allowedAuthorIds?: string[]
	/** Disallow authors with one of these user IDs. */
	disallowedAuthorIds?: string[]
	/** Require the member to have all of these role IDs. Guild-only. */
	requireRoleIds?: string[]
	/** Disallow the member if they have any of these role IDs. Guild-only. */
	bannedRoleIds?: string[]
	/** Require the member to have these permissions. Guild-only. */
	requirePermissions?: PermissionResolvable[]
}

/**
 * Which message event this listener handles.
 * @public
 */
export type MessageEventKind = 'create' | 'update' | 'delete'

type EventBase = {
	id: string
	description?: string
	priority?: number
	filter?: ListenerFilter
}

export type CreateListener = EventBase & {
	event: 'create'
	match?: (ctx: MessageContext) => boolean | Promise<boolean>
	handle: (ctx: MessageContext) => Promise<void>
}

export type UpdateListener = EventBase & {
	event: 'update'
	match?: (ctx: MessageUpdateContext) => boolean | Promise<boolean>
	handle: (ctx: MessageUpdateContext) => Promise<void>
}

export type DeleteListener = EventBase & {
	event: 'delete'
	match?: (ctx: MessageDeleteContext) => boolean | Promise<boolean>
	handle: (ctx: MessageDeleteContext) => Promise<void>
}

/** Single, unified message listener discriminated by `event`. */
export type MessageListener = CreateListener | UpdateListener | DeleteListener

/** Context for a message reaction add/remove event. @public */
export type MessageReactContext = {
	client: Client
	reaction: MessageReaction | PartialMessageReaction
	user: User | PartialUser
	now: number
	services?: Record<string, unknown>
}

/** Which reaction event this listener handles. @public */
export type MessageReactEventKind = 'reactAdd' | 'reactRemove'

export type MessageReactListener = EventBase & {
	event: MessageReactEventKind
	match?: (ctx: MessageReactContext) => boolean | Promise<boolean>
	handle: (ctx: MessageReactContext) => Promise<void>
}

/**
 * Controls pipeline behavior and lifecycle hooks.
 * @public
 */
export type MessagePipelineOptions = {
	/** If 'first', stop after the first listener handles the message; if 'all', run all that match. */
	mode?: 'first' | 'all'
	/** Optional services container passed to every listener context (e.g., { db, cache }). */
	services?: Record<string, unknown>
	/** Optional error hook called when a listener throws. */
	onError?: (
		listener: MessageListener,
		ctx: MessageContext | MessageUpdateContext | MessageDeleteContext,
		error: unknown,
	) => void | Promise<void>
	/** Optional hook invoked after a listener successfully handles a message. */
	onHandled?: (
		listener: MessageListener,
		ctx: MessageContext | MessageUpdateContext | MessageDeleteContext,
	) => void | Promise<void>
}

export type MessageReactPipelineOptions = MessagePipelineOptions

/**
 * @internal Shared filter evaluation for any context with a message-like value.
 */
function passesFilterGeneric(
	message: Message | PartialMessage,
	f: ListenerFilter | undefined,
): boolean {
	const filter = f ?? {}

	// Bots (best-effort if author is known)
	const author = (message as Message).author
	if (filter.allowBots !== true && author?.bot) return false

	// DMs vs Guilds
	const inDM = !message.guildId
	if (inDM && filter.allowDMs !== true) return false

	// Guild filter
	if (filter.guildIds && message.guildId && !filter.guildIds.includes(message.guildId)) return false

	// Channel filter
	if (filter.channelIds && message.channelId && !filter.channelIds.includes(message.channelId))
		return false

	// Author allow/deny lists
	if (author) {
		if (filter.allowedAuthorIds && !filter.allowedAuthorIds.includes(author.id)) return false
		if (filter.disallowedAuthorIds && filter.disallowedAuthorIds.includes(author.id)) return false
	} else {
		// If author constraints exist but we don't know the author, fail fast
		if (filter.allowedAuthorIds?.length || filter.disallowedAuthorIds?.length) return false
	}

	// Role/permission filters (guild-only and only when member is known)
	const member = (message as Message).member
	if (member) {
		const memberRoles = member.roles.cache
		if (filter.requireRoleIds && !filter.requireRoleIds.every((r) => memberRoles.has(r)))
			return false
		if (filter.bannedRoleIds && filter.bannedRoleIds.some((r) => memberRoles.has(r))) return false
		if (filter.requirePermissions && !member.permissions.has(filter.requirePermissions, true))
			return false
	} else if (
		filter.requireRoleIds?.length ||
		filter.bannedRoleIds?.length ||
		filter.requirePermissions?.length
	) {
		return false
	}

	return true
}

/**
 * Creates ready-to-wire handlers for message create, update, and delete events from a single listener list.
 * @public
 */
export function createMessageHandlers(
	listeners: MessageListener[],
	options?: MessagePipelineOptions,
): {
	onCreate: (message: Message) => Promise<void>
	onUpdate: (
		oldMessage: Message | PartialMessage,
		newMessage: Message | PartialMessage,
	) => Promise<void>
	onDelete: (message: Message | PartialMessage) => Promise<void>
} {
	const ordered = [...listeners].sort((a, b) => (a.priority ?? 0) - (b.priority ?? 0))

	const createList = ordered.filter((l): l is CreateListener => l.event === 'create')
	const updateList = ordered.filter((l): l is UpdateListener => l.event === 'update')
	const deleteList = ordered.filter((l): l is DeleteListener => l.event === 'delete')

	const onCreate = async (message: Message) => {
		const now = Date.now()
		const ctx: MessageContext = {
			client: message.client,
			message,
			now,
			services: options?.services,
		}
		for (const l of createList) {
			if (!passesFilterGeneric(message, l.filter)) continue
			if (l.match && !(await l.match(ctx))) continue
			try {
				await l.handle(ctx)
				if (options?.onHandled) await options.onHandled(l, ctx)
			} catch (err) {
				if (options?.onError) await options.onError(l, ctx, err)
			}
			if ((options?.mode ?? 'all') === 'first') break
		}
	}

	const onUpdate = async (
		oldMessage: Message | PartialMessage,
		newMessage: Message | PartialMessage,
	) => {
		const client = (newMessage as Message).client ?? (oldMessage as Message).client
		const now = Date.now()
		const ctx: MessageUpdateContext = {
			client,
			oldMessage,
			newMessage,
			now,
			services: options?.services,
		}
		for (const l of updateList) {
			const msg = newMessage
			if (!passesFilterGeneric(msg, l.filter)) continue
			if (l.match && !(await l.match(ctx))) continue
			try {
				await l.handle(ctx)
				if (options?.onHandled) await options.onHandled(l, ctx as any)
			} catch (err) {
				if (options?.onError) await options.onError(l, ctx as any, err)
			}
			if ((options?.mode ?? 'all') === 'first') break
		}
	}

	const onDelete = async (message: Message | PartialMessage) => {
		const now = Date.now()
		const ctx: MessageDeleteContext = {
			client: (message as Message).client,
			message,
			now,
			services: options?.services,
		}
		for (const l of deleteList) {
			if (!passesFilterGeneric(message, l.filter)) continue
			if (l.match && !(await l.match(ctx))) continue
			try {
				await l.handle(ctx)
				if (options?.onHandled) await options.onHandled(l, ctx as any)
			} catch (err) {
				if (options?.onError) await options.onError(l, ctx as any, err)
			}
			if ((options?.mode ?? 'all') === 'first') break
		}
	}

	return { onCreate, onUpdate, onDelete }
}

/**
 * Creates ready-to-wire handlers for message reaction add/remove events from a single listener list.
 * @public
 */
export function createReactionHandlers(
	listeners: MessageReactListener[],
	options?: MessageReactPipelineOptions,
): {
	onReactionAdd: (
		reaction: MessageReaction | PartialMessageReaction,
		user: User | PartialUser,
	) => Promise<void>
	onReactionRemove: (
		reaction: MessageReaction | PartialMessageReaction,
		user: User | PartialUser,
	) => Promise<void>
} {
	const ordered = [...listeners].sort((a, b) => (a.priority ?? 0) - (b.priority ?? 0))
	const addList = ordered.filter(
		(l): l is MessageReactListener & { event: 'reactAdd' } => l.event === 'reactAdd',
	)
	const removeList = ordered.filter(
		(l): l is MessageReactListener & { event: 'reactRemove' } => l.event === 'reactRemove',
	)

	const run = async (
		list: MessageReactListener[],
		reaction: MessageReaction | PartialMessageReaction,
		user: User | PartialUser,
	) => {
		// Resolve partials (reaction & message) best-effort
		try {
			if (reaction.partial) await reaction.fetch()
		} catch {
			return
		}
		const message = reaction.message
		if (message && message.partial) {
			try {
				await message.fetch()
			} catch {
				return
			}
		}
		const now = Date.now()
		const ctx: MessageReactContext = {
			client: (reaction as MessageReaction).client,
			reaction,
			user,
			now,
			services: options?.services,
		}
		for (const l of list) {
			const msg = reaction.message as Message | PartialMessage | undefined
			if (msg && !passesFilterGeneric(msg, l.filter)) continue
			if (l.match && !(await l.match(ctx))) continue
			try {
				await l.handle(ctx)
				if (options?.onHandled) await options.onHandled(l as any, ctx as any)
			} catch (err) {
				if (options?.onError) await options.onError(l as any, ctx as any, err)
			}
			if ((options?.mode ?? 'all') === 'first') break
		}
	}

	return {
		onReactionAdd: (reaction, user) => run(addList, reaction, user),
		onReactionRemove: (reaction, user) => run(removeList, reaction, user),
	}
}
