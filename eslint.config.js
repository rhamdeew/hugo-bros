import tseslint from 'typescript-eslint';
import svelte from 'eslint-plugin-svelte';
import svelteParser from 'svelte-eslint-parser';
import tsParser from '@typescript-eslint/parser';
import globals from 'globals';

export default [
  {
    ignores: ['build/', '.svelte-kit/', 'dist/', 'node_modules/', 'src-tauri/'],
  },
  ...tseslint.configs.recommended,
  ...svelte.configs['flat/recommended'],
  {
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.es2017,
        ...globals.node,
      },
    },
  },
  {
    files: ['**/*.svelte'],
    languageOptions: {
      parser: svelteParser,
      parserOptions: {
        parser: tsParser,
      },
    },
  },
  {
    rules: {
      '@typescript-eslint/no-unused-vars': [
        'warn',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_' },
      ],
      '@typescript-eslint/no-explicit-any': 'warn',
      // goto() from $app/navigation is valid client-side; this rule targets server handle hooks
      'svelte/no-navigation-without-resolve': 'off',
      // {@html} is intentionally used for markdown preview rendering
      'svelte/no-at-html-tags': 'off',
      // Map/Set used as local computation tools inside $derived, not as reactive state
      'svelte/prefer-svelte-reactivity': 'off',
    },
  },
];
