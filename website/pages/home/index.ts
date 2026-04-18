import hljs, { HLJSApi, Language } from 'highlight.js';
import { ddslLanguage } from './ddsl.highlight.js';
import 'highlight.js/styles/github-dark-dimmed.min.css';

hljs.registerLanguage('ddsl', ddslLanguage);
hljs.highlightAll();
