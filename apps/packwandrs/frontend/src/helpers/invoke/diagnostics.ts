import type { ContentRegistry, JobRecord, ValidationReport, VariantParityReport } from '../types'
import { call } from './core'

export const diagnosticsLint = () => call<ValidationReport>('diagnostics_lint')
export const diagnosticsValidate = () => call<ValidationReport>('diagnostics_validate')
export const diagnosticsParity = () => call<VariantParityReport[]>('diagnostics_parity')
export const diagnosticsContentLint = () => call<ValidationReport>('diagnostics_content_lint')
export const diagnosticsPreflight = () => call<ValidationReport>('diagnostics_preflight')
export const diagnosticsRegistries = () => call<ContentRegistry[]>('diagnostics_registries')
export const diagnosticsInstallerTest = (id: string) => call<JobRecord>('diagnostics_installer_test', { id })
