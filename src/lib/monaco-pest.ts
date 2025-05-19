import * as monaco from 'monaco-editor';
import debounce from 'lodash.debounce';

const definition = {
  defaultToken: '',
  tokenPostfix: '.pest',

  keywords: [
    'PUSH',
    'POP',
    'PEEK',
    'POP_ALL',
    'PEEK_ALL',
    'WHITESPACE',
    'COMMENT',
    'SOI',
    'EOI',
    'NEWLINE',
    'ASCII_DIGIT',
    'ASCII_NONZERO_DIGIT',
    'ASCII_BIN_DIGIT',
    'ASCII_OCT_DIGIT',
    'ASCII_HEX_DIGIT',
    'ASCII_ALPHA_LOWER',
    'ASCII_ALPHA_UPPER',
    'ASCII_ALPHA',
    'ASCII_ALPHANUMERIC',
    'ANY',
    'LETTER',
    'CASED_LETTER',
    'UPPERCASE_LETTER',
    'LOWERCASE_LETTER',
    'TITLECASE_LETTER',
    'MODIFIER_LETTER',
    'OTHER_LETTER',
    'MARK',
    'COMBINING_SPACING_MARK',
    'ENCLOSING_MARK',
    'NONSPACING_MARK',
    'NUMBER',
    'DECIMAL_NUMBER',
    'LETTER_NUMBER',
    'OTHER_NUMBER',
    'PUNCTUATION',
    'CONNECTOR_PUNCTUATION',
    'DASH_PUNCTUATION',
    'OPEN_PUNCTUATION',
    'CLOSE_PUNCTUATION',
    'INITIAL_PUNCTUATION',
    'FINAL_PUNCTUATION',
    'OTHER_PUNCTUATION',
    'SYMBOL',
    'MATH_SYMBOL',
    'CURRENCY_SYMBOL',
    'MODIFIER_SYMBOL',
    'OTHER_SYMBOL',
    'SEPARATOR',
    'SPACE_SEPARATOR',
    'LINE_SEPARATOR',
    'PARAGRAPH_SEPARATOR',
    'CONTROL',
    'FORMAT',
    'SURROGATE',
    'PRIVATE_USE',
    'UNASSIGNED',
    'ALPHABETIC',
    'BIDI_CONTROL',
    'CASE_IGNORABLE',
    'CASED',
    'CHANGES_WHEN_CASEFOLDED',
    'CHANGES_WHEN_CASEMAPPED',
    'CHANGES_WHEN_LOWERCASED',
    'CHANGES_WHEN_TITLECASED',
    'CHANGES_WHEN_UPPERCASED',
    'DASH',
    'DEFAULT_IGNORABLE_CODE_POINT',
    'DEPRECATED',
    'DIACRITIC',
    'EXTENDER',
    'GRAPHEME_BASE',
    'GRAPHEME_EXTEND',
    'GRAPHEME_LINK',
    'HEX_DIGIT',
    'HYPHEN',
    'IDS_BINARY_OPERATOR',
    'IDS_TRINARY_OPERATOR',
    'ID_CONTINUE',
    'ID_START',
    'IDEOGRAPHIC',
    'JOIN_CONTROL',
    'LOGICAL_ORDER_EXCEPTION',
    'LOWERCASE',
    'MATH',
    'NONCHARACTER_CODE_POINT',
    'OTHER_ALPHABETIC',
    'OTHER_DEFAULT_IGNORABLE_CODE_POINT',
    'OTHER_GRAPHEME_EXTEND',
    'OTHER_ID_CONTINUE',
    'OTHER_ID_START',
    'OTHER_LOWERCASE',
    'OTHER_MATH',
    'OTHER_UPPERCASE',
    'PATTERN_SYNTAX',
    'PATTERN_WHITE_SPACE',
    'PREPENDED_CONCATENATION_MARK',
    'QUOTATION_MARK',
    'RADICAL',
    'REGIONAL_INDICATOR',
    'SENTENCE_TERMINAL',
    'SOFT_DOTTED',
    'TERMINAL_PUNCTUATION',
    'UNIFIED_IDEOGRAPH',
    'UPPERCASE',
    'VARIATION_SELECTOR',
    'WHITE_SPACE',
    'XID_CONTINUE',
    'XID_START',
  ],

  operators: ['-', '..', '!', '~', '_', '@', '$', '+', '*', '?', '=', '{', '}'],

  symbols: /[=><!~?:&|+\-*\/\^%\.]+/,

  escapes: /\\(?:[btnfr"']|u\{[a-fA-F0-9]{4}\})/,

  tokenizer: {
    root: [
      // Comments
      [/(\/\/.*$)/, 'comment'],
      [/\/\*/, 'comment', '@comment'],

      // Strings
      [/"/, 'string', '@string_double'],
      [/'/, 'string', '@string_single'],

      // Tags
      [/(#[A-Za-z_][A-Za-z0-9_]*)/, 'tag'],

      // Keywords and constants
      [
        /\b[A-Z_]+\b/,
        {
          cases: {
            '@keywords': 'keyword',
            '@default': 'identifier',
          },
        },
      ],

      // Identifiers
      [/\b[A-Za-z_][A-Za-z0-9_]*\b/, 'identifier'],

      // Operators
      [
        /[-+*?=~!@${}_]/,
        {
          cases: {
            '@operators': 'operator',
            '@default': '',
          },
        },
      ],
    ],

    comment: [
      [/[^\/*]+/, 'comment'],
      [/\*\//, 'comment', '@pop'],
      [/[\/*]/, 'comment'],
    ],

    string_double: [
      [/@escapes/, 'string.escape'],
      [/"/, 'string', '@pop'],
      [/./, 'string'],
    ],

    string_single: [
      [/@escapes/, 'string.escape'],
      [/'/, 'string', '@pop'],
      [/./, 'string'],
    ],
  },
} satisfies monaco.languages.IMonarchLanguage;

export function initializePest() {
  monaco.languages.register({ id: 'pest-rs' });
  monaco.languages.setMonarchTokensProvider('pest-rs', definition);
  monaco.languages.setLanguageConfiguration('pest-rs', {
    surroundingPairs: [
      { open: '{', close: '}' },
      { open: '(', close: ')' },
      { open: '[', close: ']' },
      { open: '"', close: '"' },
      { open: "'", close: "'" },
    ],
    autoClosingPairs: [
      { open: '{', close: '}' },
      { open: '[', close: ']' },
      { open: '(', close: ')' },
      { open: "'", close: "'", notIn: ['string', 'comment'] },
      { open: '"', close: '"', notIn: ['string', 'comment'] },
    ],
    brackets: [
      ['{', '}'],
      ['(', ')'],
      ['[', ']'],
    ],
    indentationRules: {
      increaseIndentPattern: /.*{(?: *)?$/,
      decreaseIndentPattern: /.*}(?: *)?$/,
    },
  });

  monaco.languages.registerCompletionItemProvider('pest-rs', {
    provideCompletionItems: (model, position, _, __) => {
      const keywords = definition.keywords.map((k) => {
        return {
          label: k,
          kind: monaco.languages.CompletionItemKind.Keyword,
          insertText: k,
          range: new monaco.Range(
            position.lineNumber,
            position.column,
            model.getLineCount(),
            position.column,
          ),
        } satisfies monaco.languages.CompletionItem;
      });

      return {
        suggestions: keywords,
        incomplete: false
      };
    },
  });
}
