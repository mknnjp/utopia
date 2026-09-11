import { defineConfig } from 'oxlint';
import { ignorePatterns } from './oxc.config.ts';

export default defineConfig({
  ignorePatterns,
  overrides: [
    {
      files: ['**/*.test.ts', '**/tests/**/*.test.ts', '**/tests/**/*.ts'],
      // Relax type-safety rules for test files
      rules: {
        'typescript/no-unsafe-assignment': 'off',
        'typescript/no-unsafe-call': 'off',
        'typescript/no-unsafe-member-access': 'off',
        'typescript/no-unsafe-return': 'off',
        'typescript/unbound-method': 'off',
      },
    },
    {
      files: ['scripts/**/*.ts'],
      rules: {
        'eslint/no-console': 'off',
      },
    },
    {
      files: ['compatibility-tests/**/*.ts'],
      rules: {
        // k6 test scripts use console.warn for diagnostics
        'eslint/no-console': 'off',
        // k6 heavily uses JSON.parse() with type assertions
        'typescript/no-unsafe-assignment': 'off',
        'typescript/no-unsafe-member-access': 'off',
      },
    },
    {
      // Config files use `defineConfig` which returns inferred types
      files: ['oxlint.config.ts', 'oxfmt.config.ts'],
      rules: {
        'typescript/explicit-function-return-type': 'off',
      },
    },
  ],
  plugins: ['eslint', 'oxc', 'promise', 'typescript', 'unicorn'],
  rules: {
    'eslint/eqeqeq': ['error', 'always'],
    'eslint/no-implicit-coercion': 'error',
    'eslint/prefer-const': 'error',
    'eslint/prefer-object-spread': 'error',
    'oxc/bad-replace-all-arg': 'error',
    'oxc/branches-sharing-code': 'error',
    'typescript/adjacent-overload-signatures': 'error',
    'typescript/array-type': ['error', { default: 'array-simple' }],
    'typescript/ban-types': 'error',
    'typescript/consistent-generic-constructors': 'error',
    'typescript/consistent-type-imports': 'error',
    'typescript/dot-notation': 'error',
    'typescript/explicit-function-return-type': 'error',
    'typescript/no-unsafe-assignment': 'error',
    'typescript/prefer-literal-enum-member': 'error',
    'typescript/prefer-ts-expect-error': 'error',
    'typescript/restrict-plus-operands': 'error',
    'typescript/strict-boolean-expressions': 'error',
    'unicorn/catch-error-name': 'error',
    'unicorn/prefer-node-protocol': 'error',
  },
});
