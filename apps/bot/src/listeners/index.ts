import { analyzeLogs } from '@/listeners/forum/analyze-logs'
import { countMessages } from '@/listeners/global/count-messages'
import { enforceNamePolicy } from '@/listeners/global/enforce-name-policy'
import { scanForBlocklistedFiles } from '@/listeners/global/scan-for-blocklisted-files'
import { scanForRawLogs } from '@/listeners/global/scan-for-raw-logs'

import { MessageListener } from '../types'
import { greetCommunitySupport } from './forum/greet'
import { lockOnOpDeletesStarter } from './forum/lock-on-op-delete-starter'
import { remindSolvedCreate, remindSolvedUpdate } from './forum/solved-reminder'

const listeners: MessageListener[] = [
	greetCommunitySupport,
	remindSolvedCreate,
	remindSolvedUpdate,
	lockOnOpDeletesStarter,
	countMessages,
	scanForBlocklistedFiles,
	enforceNamePolicy,
	scanForRawLogs,
	analyzeLogs,
]

export default listeners
