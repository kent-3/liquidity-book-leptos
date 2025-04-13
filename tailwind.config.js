const defaultTheme = require("tailwindcss/defaultTheme");

/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: "media", // or 'class' if you want to toggle with a class
  content: {
    files: ["./app/*.html", "./app/src/**/*.rs"],
  },
  theme: {
    extend: {
      animation: {
        "ping-once": "ping 1s cubic-bezier(0, 0, 0.2, 1) 1",
        "spin-slow": "spin 3s linear infinite",
        "spin-medium": "spin 1.5s linear infinite",
      },
      boxShadow: {
        "gold-glow":
          "0 0 8px rgba(255, 191, 90, 0.6), 0 0 16px rgba(255, 191, 90, 0.4), 0 0 32px rgba(255, 191, 90, 0.2)",
        // "foam-glow":
        //   "0 0 6px rgba(156, 207, 216, 0.6), 0 0 12px rgba(156, 207, 216, 0.4), 0 0 18px rgba(156, 207, 216, 0.2)",
        "foam-glow": "0 0 6px rgba(156, 207, 216, 1)", // Single-layer shadow for glowing effect
      },
      dropShadow: {
        "foam-glow": "0 0 6px rgba(156, 207, 216, 0.9)", // Simple glow effect
      },
      colors: {
        // base: "#191724",
        // surface: "#1f1d2e",
        // overlay: "#26233a",
        muted: "#6e6a86",
        subtle: "#908caa",
        text: "#e0def4",
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
        border: "oklch(var(--border))",
        input: "oklch(var(--input))",
        ring: "oklch(var(--ring))",
        background: "oklch(var(--background))",
        foreground: "oklch(var(--foreground))",
        primary: {
          DEFAULT: "oklch(var(--primary))",
          foreground: "oklch(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "oklch(var(--secondary))",
          foreground: "oklch(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "oklch(var(--destructive))",
          foreground: "oklch(var(--destructive-foreground))",
        },
        muted: {
          DEFAULT: "oklch(var(--muted))",
          foreground: "oklch(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "oklch(var(--accent))",
          foreground: "oklch(var(--accent-foreground))",
        },
        popover: {
          DEFAULT: "oklch(var(--popover))",
          foreground: "oklch(var(--popover-foreground))",
        },
        card: {
          DEFAULT: "oklch(var(--card))",
          foreground: "oklch(var(--card-foreground))",
        },
      },
      borderRadius: {
        xl: "calc(var(--radius) + 4px)",
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
        xs: "2px",
      },
      transitionTimingFunction: {
        standard: "cubic-bezier(0.2, 0, 0, 1)",
        "standard-decelerate": "cubic-bezier(0, 0, 0, 1)",
        "standard-accelerate": "cubic-bezier(0.3, 0.1, 1, 1)",
        "emphasized-decelerate": "cubic-bezier(0.05, 0.7, 0.1, 1.0)",
        "emphasized-accelerate": "cubic-bezier(0.3, 0.0, 0.8, 0.15)",
      },
      fontFamily: {
        sans: ["Inter", ...defaultTheme.fontFamily.sans],
      },
    },
  },

  plugins: [],
  // plugins: [require("tailwindcss-animate")],
};
