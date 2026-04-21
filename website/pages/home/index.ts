import hljs from 'highlight.js/lib/core';
import { ddslLanguage } from './ddsl.highlight.js';
import defineRust from 'highlight.js/lib/languages/rust';
import 'highlight.js/styles/github-dark-dimmed.min.css';

hljs.registerLanguage('ddsl', ddslLanguage);
hljs.registerLanguage('rust', defineRust);
hljs.highlightAll();
