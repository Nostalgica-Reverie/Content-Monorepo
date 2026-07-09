import { drizzle } from 'drizzle-orm/node-postgres'
import { Pool } from 'pg'

import * as schema from './schema'

const pool = new Pool({ connectionString: process.env.POSTGRES_DB_URL! })
export const db = drizzle(pool, { schema })
export type DB = typeof db
