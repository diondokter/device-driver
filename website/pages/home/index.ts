import hljs from 'highlight.js/lib/core';
import { ddslLanguage } from './ddsl.highlight.js';
import defineRust from 'highlight.js/lib/languages/rust';

hljs.registerLanguage('ddsl', ddslLanguage);
hljs.registerLanguage('rust', defineRust);
hljs.highlightAll();
