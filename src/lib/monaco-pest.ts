import * as monaco from 'monaco-editor';
import { commands } from './bindings';

const pestId = 'pest-rs';

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

  escapes: /\\(?:[abfnrtv\\"']|x[0-9A-Fa-f]{1,4}|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8})/,

  tokenizer: {
    root: [
      // Comments
      [/(\/\/.*$)/, 'comment'],
      [/\/\*/, 'comment', '@comment'],

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

      // Strings
      [/"([^"\\]|\\.)*$/, 'string.invalid'], // non-teminated string
      [/"/, { token: 'string.quote', bracket: '@open', next: '@string' }],

      // characters
      [/'[^\\']'/, 'string'],
      [/(')(@escapes)(')/, ['string', 'string.escape', 'string']],
      [/'/, 'string.invalid'],
    ],

    comment: [
      [/[^\/*]+/, 'comment'],
      [/\*\//, 'comment', '@pop'],
      [/[\/*]/, 'comment'],
    ],

    string: [
      [/[^\\"]+/, 'string'],
      [/@escapes/, 'string.escape'],
      [/\\./, 'string.escape.invalid'],
      [/"/, { token: 'string.quote', bracket: '@close', next: '@pop' }],
    ],
  },
} satisfies monaco.languages.IMonarchLanguage;

async function findReferences(
  model: monaco.editor.ITextModel,
  position: monaco.Position,
): Promise<monaco.languages.Location[] | undefined> {
  const word = model.getWordAtPosition(position);
  if (!word) {
    return undefined;
  }

  const references = await commands.findRuleReferences(word.word);
  if (!references) {
    return undefined;
  }
  const formattedReferences: monaco.languages.Location[] = references.map((r) => {
    return {
      range: {
        startLineNumber: r.start_line,
        startColumn: r.start_col,
        endLineNumber: r.end_line,
        endColumn: r.end_col,
      },
      uri: model.uri,
    };
  });

  return formattedReferences;
}

export function initializePest() {
  monaco.languages.register({ id: pestId });
  monaco.languages.setMonarchTokensProvider(pestId, definition);
  monaco.languages.setLanguageConfiguration(pestId, {
    surroundingPairs: [
      { open: '{', close: '}' },
      { open: '(', close: ')' },
      { open: '[', close: ']' },
      { open: '"', close: '"' },
      { open: "'", close: "'" }
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

  monaco.languages.registerCompletionItemProvider(pestId, {
    provideCompletionItems: async (model, position, _, __) => {
      const word = model.getWordUntilPosition(position);
      const unfilteredVars = await commands.getAllRules();

      const variables: monaco.languages.CompletionItem[] = unfilteredVars
        .filter((v) => !definition.keywords.includes(v))
        .map((v) => {
          return {
            label: v,
            kind: monaco.languages.CompletionItemKind.Variable,
            insertText: v,
            range: new monaco.Range(
              position.lineNumber,
              word.startColumn,
              position.lineNumber,
              word.endColumn,
            ),
          } satisfies monaco.languages.CompletionItem;
        });

      const keywords: monaco.languages.CompletionItem[] = definition.keywords.map((k) => {
        return {
          label: k,
          kind: monaco.languages.CompletionItemKind.Keyword,
          insertText: k,
          range: new monaco.Range(
            position.lineNumber,
            word.startColumn,
            position.lineNumber,
            word.endColumn,
          ),
        } satisfies monaco.languages.CompletionItem;
      });

      return {
        suggestions: keywords.concat(variables),
        incomplete: false,
      };
    },
  });

  monaco.languages.registerDefinitionProvider(pestId, {
    provideDefinition: findReferences,
  });

  monaco.languages.registerRenameProvider(pestId, {
    provideRenameEdits: async (model, position, newName) => {
      const word = model.getWordAtPosition(position);
      console.log(newName);

      if (!word) {
        return {
          rejectReason: 'No text found',
          edits: [],
        };
      }

      if (definition.keywords.includes(word.word)) {
        return {
          rejectReason: 'Cannot edit built-in rules',
          edits: [],
        };
      }

      const references = await findReferences(model, position);
      if (!references) {
        return {
          rejectReason: 'No rules found',
          edits: [],
        };
      }

      const edits: monaco.languages.IWorkspaceTextEdit[] = references?.map((v) => {
        return {
          resource: model.uri,
          textEdit: {
            range: v.range,
            text: newName,
          },
          versionId: undefined,
        } satisfies monaco.languages.IWorkspaceTextEdit;
      });

      return { edits };
    },

    resolveRenameLocation: async (model, position) => {
      const word = model.getWordAtPosition(position);

      if (!word) {
        return {
          rejectReason: 'You cannot rename this element',
          text: '',
          range: new monaco.Range(
            position.lineNumber,
            position.column,
            position.lineNumber,
            position.column,
          ),
        };
      }

      if (definition.keywords.includes(word.word)) {
        return {
          rejectReason: 'You cannot rename this element',
          text: word.word,
          range: new monaco.Range(
            position.lineNumber,
            word.startColumn,
            position.lineNumber,
            word.endColumn,
          ),
        };
      }

      const references = await findReferences(model, position);
      if (!references) {
        return {
          rejectReason: 'You cannot rename this element',
          text: word.word,
          range: new monaco.Range(
            position.lineNumber,
            word.startColumn,
            position.lineNumber,
            word.endColumn,
          ),
        };
      }
    },
  });
}
