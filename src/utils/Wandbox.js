import https from 'https';
import fetch from 'node-fetch';
import { Collection } from 'discord.js'

/**
 * A class designed to fetch & hold the list of valid
 * compilers from wandbox.
 */
export class Compilers extends Collection {
    /**
     * Creates a Compilers object.
     *
     * @param {CompilerClient} client compiler client for events
     */
    constructor(client) {
        super();

        // List of compilers WandBox has set up incorrectly and need to be ignored to prevent backend environmental setup errors.
        this.brokencompilers = ['ghc-head'];

        this.client = client;
    }

    /**
     * Asyncronously fetches the list of valid compilers from wandbox and populates cache.
     * Note: This can throw
     *
     * @return {type}  description
     */
    async initialize() {
        let compilers = null;
        try {
            const response = await fetch("https://wandbox.org/api/list.json");
            const json = await response.json();
            compilers = json;
        }
        catch (error) {
            throw (error); // throw it up
        }

        compilers.forEach((obj) => {
            const lang = obj.language.toLowerCase();
            const compiler = obj.name;

            // Skip any broken compilers on WandBox so users cannot accidentally use them.
            if (this.brokencompilers.includes(compiler)) {
                return;
            }

            // if the language hasn't been mapped yet, do it
            if (!this.has(lang)) {
                this.set(lang, []);
            }

            // add valid compiler to our map
            this.get(lang).push(compiler);
        });

        // dont emit under testing conditions
        if (this.client)
            this.client.emit('compilersReady');
    }


    /**
     * Grabs a list of compilers for a given language from the cache
     *
     * @param  {string} language wandbox language to fetch
     * @return {array}          array of compilers
     */
    getCompilers(language) {
        return this.get(language);
    }


    /**
     * Determines if the input compiler is a valid compiler in our cache
     *
     * @param  {string} compiler string to search for
     * @return {boolean}          true upon complier found
     */
    isValidCompiler(compiler) {

        let found = false;
        this.forEach((value, key, map) => {
            if (value.includes(compiler))
                found = true;
        })

        return found;
    }
}

/**
 * Class which represents all the settings and information for a single compilation
 * request. This should be built and used in coordination with Compiler.
 */
export class CompileSetup {
    /**
     * Creates a compilation setup for usage with the Compiler object.
     * You may pass a language instead of a compiler for the second parameter,
     * and it will be compiled with the first compiler found in the list. The compiler
     * used is #1 on the menu for ;compilers <lang>.
     * @param {String} code
     * @param {String} compiler
     * @param {String} stdin
     * @param {Boolean} save
     * @param {string} compiler_option_raw
     * @param {Compilers} compilers
     */
    constructor(code, compiler, stdin, save, compiler_option_raw, compilers) {
        this.code = code;
        this.stdin = stdin;
        this.save = save;
        this.compiler_option_raw = compiler_option_raw.split(' ').join('\n'); // joined by comma per doc spec

        let comp = compiler.toLowerCase();
        if (compilers.has(comp)) // if lang instead of raw compiler
            this.compiler = compilers.get(comp)[0];
        else
            this.compiler = comp;
    }
}

/**
 * Request sender which creates and sends a CompileSetup
 */
export class Compiler {
    /**
     * Creates a compilation object which compiles code.
     *
     * @param {CompileSetup} compilesetup
     */
    constructor(compilesetup) {
        this.compilesetup = compilesetup
    }

    /**
     * Asyncronously sends a request to wandbox servers with the code, language, and compiler.
     * Note: This can throw
     */
    async compile() {
        try {
            const response = await fetch("https://wandbox.org/api/compile.json", {
                method: "POST",
                body: JSON.stringify(this.compilesetup).replace('compiler_option_raw', 'compiler-option-raw'),
                headers: {
                    'Content-Type': 'application/json; charset=utf-8'
                },
            });
            const json = await response.json();
            return json;    
        }
        catch (error) {
            throw(error); // rethrow to higher level
        }
    }
}
