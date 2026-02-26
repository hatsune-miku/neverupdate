module.exports = {
  plugins: ['@trivago/prettier-plugin-sort-imports'],
  printWidth: 200,
  tabWidth: 2,
  singleQuote: true,
  semi: false,
  importOrder: [
    '^react(-.*)?$',
    '^[a-zA-Z]',
    '^@',
    '^@[a-zA-Z]',
    '^\\./(?!.*\\.(css|scss)$).*',
    '^\\.\\./(?!.*\\.(css|scss)$).*',
    '\\.(css|scss)$',
  ],
  importOrderSeparation: true,
  importOrderSortSpecifiers: true,
}
