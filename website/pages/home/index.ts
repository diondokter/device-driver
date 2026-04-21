import hljs from 'highlight.js/lib/core';
import { ddslLanguage } from './ddsl.highlight.js';
import 'highlight.js/styles/github-dark-dimmed.min.css';

hljs.registerLanguage('ddsl', ddslLanguage);
hljs.highlightAll();
