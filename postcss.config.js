const postcssAtImport = require("postcss-import")
const postcssJitProps = require('postcss-jit-props');
const postcssCustomMedia = require('postcss-custom-media');

const OpenProps = require('open-props');

module.exports = {
    plugins: [
        postcssAtImport(),
        // only vars used are in build output
        postcssCustomMedia(),
        postcssJitProps(OpenProps),
    ],
};
