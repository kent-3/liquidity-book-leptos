/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: "media", // or 'class' if you want to toggle with a class
  content: {
    files: ["*.html", "./src/**/*.rs"],
  },
  theme: {
    extend: {
      animation: {
        "ping-once": "ping 1s cubic-bezier(0, 0, 0.2, 1) 1",
        "spin-slow": "spin 3s linear infinite",
      },
      colors: {
        // base: "#191724",
        // surface: "#1f1d2e",
        // overlay: "#26233a",
        muted: "#6e6a86",
        subtle: "#908caa",
        // text: "#e0def4",
        love: "#eb6f92",
        gold: "#f6c177",
        rose: "#ebbcba",
        pine: "#31748f",
        foam: "#9ccfd8",
        iris: "#c4a7e7",
        highlight: {
          low: "#21202e",
          med: "#403d52",
          high: "#524f67",
        },
      },
      transitionTimingFunction: {
        standard: "cubic-bezier(0.2, 0, 0, 1)",
        "standard-decelerate": "cubic-bezier(0, 0, 0, 1)",
        "standard-accelerate": "cubic-bezier(0.3, 0.1, 1, 1)",
        "emphasized-decelerate": "cubic-bezier(0.05, 0.7, 0.1, 1.0)",
        "emphasized-accelerate": "cubic-bezier(0.3, 0.0, 0.8, 0.15)",
      },
    },
  },

  plugins: [],
};
