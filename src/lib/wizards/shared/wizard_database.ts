/**
 * Wizard Database Connection Configuration
 *
 * Represents the connection parameters for the wizard database features.
 * Used by all wizard backend functions to query data (interfaces, patients, visits).
 *
 * Note: The wizards currently return sample data and do not require an actual
 * database connection. These settings are retained for compatibility.
 *
 * @property host - Database server hostname or IP address
 * @property port - Database port (typically 1433 for SQL Server)
 * @property database - Database name
 * @property user - Database username
 * @property password - Database password
 */
export interface WizardDatabase {
  host: string,
  port: number,
  database: string,
  user: string,
  password: string,
}
