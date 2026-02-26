import pluginReact from 'eslint-plugin-react'
import { defineConfig } from 'eslint/config'
import globals from 'globals'
import tseslint from 'typescript-eslint'

import js from '@eslint/js'

/**
 * 同时对 js 和 ts 生效的配置
 *
 * @type {import('typescript-eslint').ConfigWithExtends}
 */
const commonConfigs = {
  extends: ['js/recommended'],
  languageOptions: {
    globals: {
      ...globals.browser,
      ...globals.node,
      ...globals.es2021,
    },
  },
  plugins: { js },
}

/**
 * 同时对 js 和 ts 生效的规则
 *
 * @type {Partial<import('@rspack/core/dist/builtin-plugin').Rules>}
 */
const commonRules = {
  'ban-ts-comment': 'off',
  '@typescript-eslint/no-explicit-any': 'off',

  /** 允许在js里面按约定、无声明地解构props */
  'react/prop-types': 'off',

  /** 让用@ts-ignore */
  '@typescript-eslint/ban-ts-comment': 'off',

  /** 允许定义空的 interface */
  '@typescript-eslint/no-empty-object-type': 'off',

  /** 可以用 require() 语句导入非源文件的文件比如 less, svg 等 */
  '@typescript-eslint/no-require-imports': 'off',
}

/**
 * 只对 js 生效的规则
 *
 * @type {Partial<import('@rspack/core/dist/builtin-plugin').Rules>}
 */
const jsRules = {
  'react/display-name': 'off',
}

/**
 * 只对 ts 生效的规则
 *
 * @type {Partial<import('@rspack/core/dist/builtin-plugin').Rules>}
 */
const tsRules = {
  /** eslint 对函数声明里面的形参有假阳性报错，关闭检查，用tsconfig代替检查 */
  '@typescript-eslint/no-unused-vars': 'off',

  /** 允许 `onClose && onClose()` 写法 */
  '@typescript-eslint/no-unused-expressions': 'off',
  'no-unused-vars': 'off',
}

export default defineConfig([
  tseslint.configs.recommended,
  pluginReact.configs.flat.recommended,
  {
    files: ['**/*.{ts,mts,cts,tsx}'],
    ...commonConfigs,
    rules: {
      ...commonRules,
      ...tsRules,
    },
  },
  {
    files: ['**/*.{js,mjs,cjs,jsx}'],
    ...commonConfigs,
    rules: {
      ...commonRules,
      ...jsRules,
    },
  },
])
