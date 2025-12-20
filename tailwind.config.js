/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // Discord-inspired dark theme
        'discord-dark': '#313338',
        'discord-darker': '#2B2D31',
        'discord-darkest': '#1E1F22',
        'discord-primary': '#5865F2',
        'discord-primary-hover': '#4752C4',
        'discord-success': '#23A559',
        'discord-danger': '#DA373C',
        'discord-warning': '#F0B232',
        'discord-text': '#DBDEE1',
        'discord-text-muted': '#B5BAC1',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0', transform: 'translateY(-4px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        toastEnter: {
          '0%': { opacity: '0', transform: 'translate(-50%, 96px)' },
          '100%': { opacity: '1', transform: 'translate(-50%, 0)' },
        }
      },
      animation: {
        fadeIn: 'fadeIn 0.3s ease-out',
        toastEnter: 'toastEnter 0.3s ease-out',
      }
    },
  },
  plugins: [],
}
