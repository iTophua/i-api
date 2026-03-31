import { invoke } from '@tauri-apps/api/core'

export interface SecretStorage {
  storeSecret(resourceId: string, secretType: string, key: string, value: string): Promise<void>
  retrieveSecret(resourceId: string, secretType: string, key: string): Promise<string | null>
  deleteSecret(resourceId: string, secretType: string, key: string): Promise<void>
}

export function useSecretStorage(): SecretStorage {
  const storeSecret = async (
    resourceId: string,
    secretType: string,
    key: string,
    value: string
  ): Promise<void> => {
    await invoke('store_secret', { resourceId, secretType, key, value })
  }

  const retrieveSecret = async (
    resourceId: string,
    secretType: string,
    key: string
  ): Promise<string | null> => {
    return await invoke<string | null>('retrieve_secret', { resourceId, secretType, key })
  }

  const deleteSecret = async (
    resourceId: string,
    secretType: string,
    key: string
  ): Promise<void> => {
    await invoke('delete_secret', { resourceId, secretType, key })
  }

  return {
    storeSecret,
    retrieveSecret,
    deleteSecret,
  }
}

export function useAuthSecrets() {
  const { storeSecret, retrieveSecret, deleteSecret } = useSecretStorage()

  const storeBearerToken = async (requestId: string, token: string): Promise<void> => {
    await storeSecret(requestId, 'bearer', 'token', token)
  }

  const retrieveBearerToken = async (requestId: string): Promise<string | null> => {
    return await retrieveSecret(requestId, 'bearer', 'token')
  }

  const deleteBearerToken = async (requestId: string): Promise<void> => {
    await deleteSecret(requestId, 'bearer', 'token')
  }

  const storeBasicAuth = async (
    requestId: string,
    username: string,
    password: string
  ): Promise<void> => {
    await storeSecret(requestId, 'basic', 'username', username)
    await storeSecret(requestId, 'basic', 'password', password)
  }

  const retrieveBasicAuth = async (
    requestId: string
  ): Promise<{ username: string; password: string } | null> => {
    const username = await retrieveSecret(requestId, 'basic', 'username')
    const password = await retrieveSecret(requestId, 'basic', 'password')

    if (username && password) {
      return { username, password }
    }
    return null
  }

  const deleteBasicAuth = async (requestId: string): Promise<void> => {
    await deleteSecret(requestId, 'basic', 'username')
    await deleteSecret(requestId, 'basic', 'password')
  }

  const storeApiKey = async (requestId: string, key: string, value: string): Promise<void> => {
    await storeSecret(requestId, 'apikey', key, value)
  }

  const retrieveApiKey = async (requestId: string, key: string): Promise<string | null> => {
    return await retrieveSecret(requestId, 'apikey', key)
  }

  const deleteApiKey = async (requestId: string, key: string): Promise<void> => {
    await deleteSecret(requestId, 'apikey', key)
  }

  const storeEnvironmentSecret = async (
    environmentId: string,
    key: string,
    value: string
  ): Promise<void> => {
    await storeSecret(environmentId, 'env', key, value)
  }

  const retrieveEnvironmentSecret = async (
    environmentId: string,
    key: string
  ): Promise<string | null> => {
    return await retrieveSecret(environmentId, 'env', key)
  }

  const deleteEnvironmentSecret = async (
    environmentId: string,
    key: string
  ): Promise<void> => {
    await deleteSecret(environmentId, 'env', key)
  }

  return {
    storeBearerToken,
    retrieveBearerToken,
    deleteBearerToken,
    storeBasicAuth,
    retrieveBasicAuth,
    deleteBasicAuth,
    storeApiKey,
    retrieveApiKey,
    deleteApiKey,
    storeEnvironmentSecret,
    retrieveEnvironmentSecret,
    deleteEnvironmentSecret,
  }
}

export function maskSensitiveValue(value: string): string {
  if (!value || value.length <= 4) {
    return '*'.repeat(value?.length || 0)
  }
  const visible = value.slice(0, 2)
  const masked = '*'.repeat(value.length - 4)
  const end = value.slice(-2)
  return `${visible}${masked}${end}`
}

export const SENSITIVE_FIELDS = {
  bearer: ['token'],
  basic: ['password'],
  apikey: ['value'],
}

export function isSensitiveField(authType: string, field: string): boolean {
  const fields = SENSITIVE_FIELDS[authType as keyof typeof SENSITIVE_FIELDS]
  return fields?.includes(field) ?? false
}
