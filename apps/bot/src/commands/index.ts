import { applyCommand } from '@/commands/apply'
import { approveCommand } from '@/commands/approve'
import { assignCommand } from '@/commands/assign'
import { memberCommand } from '@/commands/member'
import { pmCommand } from '@/commands/pm'
import { rejectCommand } from '@/commands/reject'
import { resetCommand } from '@/commands/reset'
import type { AnyCommand } from '@/types/commands'

import { docsCommand } from './docs'
import { githubCommand } from './github'
import { nukeCommand } from './nuke'
import { pingCommand } from './ping'
import { solvedCommand } from './solved'
import { verifyCommand } from './verify'
import { watchlistCommand } from './watchlist'

export const commands: AnyCommand[] = [
	docsCommand,
	githubCommand,
	pingCommand,
	solvedCommand,
	verifyCommand,
	resetCommand,
	pmCommand,
	memberCommand,
	// reportCommand,
	applyCommand,
	assignCommand,
	approveCommand,
	rejectCommand,
	watchlistCommand,
	nukeCommand,
]

export default commands
