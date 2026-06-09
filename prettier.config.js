/** @type {import('prettier').Config & import('prettier-plugin-tailwindcss').PluginOptions} */
export default {
  plugins: ["prettier-plugin-tailwindcss"],
  useTabs: true,
  tabWidth: 2,
  singleQuote: false,
  arrowParens: "always",
  trailingComma: "none",
  semi: true,
  endOfLine: "auto",
  printWidth: 120
}