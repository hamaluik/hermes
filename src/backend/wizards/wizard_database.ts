/**
 * HL7 system Database Connection Configuration
 *
 * Represents the connection parameters needed to connect to a HL7 system
 * PostgreSQL database. Used by all wizard backend functions to query
 * production data (interfaces, patients, visits).
 *
 * Security Note:
 * Database credentials are stored in user settings (via Tauri store plugin)
 * and persisted locally. They are never sent to external services - only
 * used for direct database connections from the Tauri backend.
 *
 * @property host - Database server hostname or IP address
 * @property port - PostgreSQL port (typically 5432)
 * @property database - Database name (e.g., "wizarddb")
 * @property user - Database username
 * @property password - Database password (stored securely in settings)
 */
export interface WizardDatabase {
  host: string,
  port: number,
  database: string,
  user: string,
  password: string,
}
