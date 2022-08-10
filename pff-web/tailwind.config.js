/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./www/**/*.{html,js}'],
  theme: {
    extend: {},
  },
  plugins: [require('@tailwindcss/forms')],
};
