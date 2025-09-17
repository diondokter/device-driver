// Copy of https://github.com/kdl-org/kdl-org.github.io/blob/main/static/play/kdl.monarch.js
// MIT License
// 
// Copyright (c) 2024 Danielle Smith
// 
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

function concat(a, b) {
  if (!a) return b
  if (!b) return a
  return `${a}.${b}`
}

function makeStrings(prefix = '') {
  return [
    [/(#*)"/, { token: concat(prefix, 'string.quote'), bracket: '@open', next: `@${prefix}stringraw.\$1` }],
    [/"/, { token: concat(prefix, 'string.quote'), bracket: '@open', next: `@${prefix}string` }],
  ]
}

function makeString(prefix = '') {
  return  [
    [/[^\\"]+/, concat(prefix, 'string')],
    [/@escapes/, concat(prefix, 'string.escape')],
    [/\\./, concat(prefix, 'string.escape.invalid')],
    [/"/, { token: concat(prefix, 'string.quote'), bracket: '@close', next: '@pop' }]
  ]
}

function makeStringraw(prefix = '') {
  return [
    [/[^"#]+/, { token: concat(prefix, 'string') }],
    [
      /"(#*)/,
      {
        cases: {
          '$1==$S2': { token: concat(prefix, 'string.quote'), bracket: '@close', next: '@pop' },
          '@default': { token: concat(prefix, 'string') }
        }
      }
    ],
    [/["#]/, { token: concat(prefix, 'string') }]
  ]
}

export const config = {
  comment: {
    lineComment: '//',
    blockComment: ['/*', '*/']
  },
  brackets: [
    ['(', ')'],
    ['{', '}']
  ],
  surroundingPairs: [
    { open: '{', close: '}' },
    { open: '(', close: ')' },
    { open: '"', close: '"' }
  ],
  autoClosingPairs: [
    { open: '{', close: '}' },
    { open: '(', close: ')' },
    { open: '"', close: '"', notIn: ['string'] }
  ]
}

export const language = {
	tokenPostfix: '.kdl',
  defaultToken: 'invalid',

  keywords: [
    'null', 'true', 'false'
  ],

  typeKeywords: [
  ],

  operators: [
    '='
  ],

  brackets: [ ['{','}','delimiter.curly'], ['(',')','delimiter.parenthesis'] ],

  escapes: /\\([nrt0\"''\\]|x\h{2}|u\{\h{1,6}\})/,
	intSuffixes: /[iu](8|16|32|64|128|size)/,
	floatSuffixes: /f(32|64)/,

  tokenizer: {
    root: [
			{ include: '@constant' },
			{ include: '@numbers' },
			{ include: '@strings' },
			{ include: '@whitespace' },
      [/\(/, { token: '@brackets', next: '@type_annotation' }],
      [/\/-\{/, { token: 'comment.slashdash', next: '@slashdash_children' }],
      [/\/-/, { token: 'comment.slashdash', next: '@slashdash' }],
      [
        /(?![\\{\}<>;\[\]\=,\(\)\s])([\u0021-\uFFFF]+)(=)/,
        ['identifier', 'operator']
      ],
      [
        /(?![\\{\}<>;\[\]\=,\(\)\s])[\u0021-\uFFFF]+/,
        'identifier'
      ],
      [/=/, 'operator'],
      [/;/, 'delimiter.semicolon'],
      [/\\/, 'delimiter.line-break'],
      [/[{}()]/, '@brackets'],
    ],
		whitespace: [
			[/[ \t\r\n]+/, 'white'],
			[/\/\*/, 'comment', '@comment'],
			[/\/\/.*$/, 'comment']
		],
		comment: [
			[/[^\/*]+/, 'comment'],
			[/\/\*/, 'comment', '@push'],
			['\\*/', 'comment', '@pop'],
			[/[\/*]/, 'comment']
		],
    constant: [
      [/#null/, 'keyword.constant.null'],
      [/#true|#false/, 'keyword.constant.boolean'],
      [/#-?inf|#nan/, 'number.float.constant']
    ],
		numbers: [
      [/(0o[0-7_]+)(@intSuffixes)?/, 'number.octal'],
      [/(0b[0-1_]+)(@intSuffixes)?/, 'number.binary'],
      [/[\d][\d_]*(\.[\d][\d_]*)?[eE][+-][\d_]+(@floatSuffixes)?/, 'number.float'],
      [/\b(\d\.?[\d_]*)(@floatSuffixes)?\b/, 'number.float'],
      [/(0x[\da-fA-F]+)_?(@intSuffixes)?/, 'number.hex'],
      [/[\d][\d_]*(@intSuffixes?)?/, 'number.integer']
    ],
    strings: makeStrings(),
		string: makeString(),
    stringraw: makeStringraw(),
    commentstrings: makeStrings('comment'),
		commentstring: makeString('comment'),
    commentstringraw: makeStringraw('comment'),
    type_annotation: [
      { include: '@strings' },
      [
        /(?![\\{\}<>;\[\]\=,\(\)\s])[\u0021-\u0028\u0030-\uFFFF]+/,
        'type.identifier'
      ],
      [/\)/, { token: '@brackets', next: '@pop' }]
    ],
    slashdash: [
      { include: '@commentstrings' },
      [/[=;()]|\S/, 'comment.slashdash'],
      [/\s/, { token: 'white', next: '@pop' }]
    ],
    slashdash_children: [
      { include: '@whitespace' },
      [/[=;()]/, 'comment.slashdash'],
      { include: '@commentstrings' },
      [/[^{}]+/, 'comment.slashdash'],
      [/\{/, { token: 'comment.slashdash', bracket: '@open', next: '@push' }],
      [/\}/, { token: 'comment.slashdash', bracket: '@close', next: '@pop' }]
    ]
  },
}