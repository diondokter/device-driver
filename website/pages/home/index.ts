import hljs from 'highlight.js';
import rust from 'highlight.js/lib/languages/rust';
import 'highlight.js/styles/github-dark.min.css';

hljs.registerLanguage('rust', rust);
hljs.highlightAll();
