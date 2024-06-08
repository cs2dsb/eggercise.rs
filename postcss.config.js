const postcssJitProps = require('postcss-jit-props');

const atImport = require("postcss-import")
const OpenProps = require('open-props');

module.exports = {
    plugins: [
        atImport,
        // only vars used are in build output
        postcssJitProps(OpenProps),
    ],
};
