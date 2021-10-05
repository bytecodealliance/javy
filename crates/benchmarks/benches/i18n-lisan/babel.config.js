const plugins = [];
const presets = [];

if (process.env["NODE_ENV"] === "test") {
  presets.push("@babel/preset-env");
  presets.push("@babel/preset-typescript");
}

module.exports = {
  presets,
  plugins
};
