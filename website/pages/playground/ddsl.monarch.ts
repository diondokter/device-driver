import * as monaco from 'monaco-editor';

export const config: monaco.languages.LanguageConfiguration = {
  comments: {
    lineComment: '//',
  },
  brackets: [
    ['[', ']'],
    ['{', '}']
  ],
  surroundingPairs: [
    { open: '{', close: '}' },
    { open: '[', close: ']' },
    { open: '"', close: '"' }
  ],
  autoClosingPairs: [
    { open: '{', close: '}' },
    { open: '[', close: ']' },
    { open: '"', close: '"' }
  ]
};

export const language: monaco.languages.IMonarchLanguage = {
  // Set defaultToken to invalid to see what you do not tokenize yet
  defaultToken: 'invalid',

  nodeTypes: [
    'manifest', 'device', 'register', 'command', 'buffer', 'block', 'extern', 'enum', 'field', 'fieldset'
  ],

  keywords: [
    'default', 'catch-all', 'allow', 'as'
  ],

  typeKeywords: [
    'bool', 'int', 'uint', 'u8', 'u16', 'u32', 'u64', 'i8', 'i16', 'i32', 'i64', 'LE', 'BE', '_', 'RO', 'RW', 'WO', 'mapped', 'indexed'
  ],

  // The main tokenizer for our languages
  tokenizer: {
    root: [
      // numbers
      [/-?0x[_0-9a-fA-F]+/, 'number'],
      [/-?0o[_0-7]+/, 'number'],
      [/-?0b[_0-1]+/, 'number'],
      [/-?[0-9][_0-9]*/, 'number'],

      [/[\w][\w-]*:/, 'variable.name'],

      // identifiers and keywords
      [/[\w][\w-]*/, {
        cases: {
          '@nodeTypes': 'type',
          '@typeKeywords': 'keyword',
          '@keywords': 'keyword',
          '@default': 'identifier'
        }
      }],

      // whitespace
      { include: '@whitespace' },

      // delimiters and operators
      [/[{}\[\]]/, 'delimiter'],
      [/[->*]/, 'operator'],


      // delimiter: after number because of .\d floats
      [/[:,]/, 'delimiter'],

      // strings
      [/"[^"]*"/, 'string'],
    ],

    whitespace: [
      [/[ \t\r\n]+/, 'white'],
      [/\/\/.*$/, 'comment'],
    ],
  },
};