//! The stable codes of errors and warnings, each with what `mitra explain` says.

/// Every code, with its headline and its long explanation.
pub(super) const CODES: [(&str, &str, &str); 15] = [
    (
        "M001",
        "evidence refutes a claim",
        "A claim says a function is correct, or equal to a definition, on some\n\
         inputs, and a piece of evidence says otherwise on one of them. The\n\
         evidence may refute the claim directly, or through a class: with a\n\
         `uniform` claim that is vouched for, one failing input stands for its\n\
         whole class.\n\n\
         One piece is wrong. Mitra lists the code of the function, the code\n\
         of what it uses, what the model states (the claim, an assumption, a\n\
         `uses` clause) and each piece of evidence, with how plausible it is\n\
         that each is the wrong one. A piece trusted at 1 can't be the wrong\n\
         one.\n\n\
         On a requirement (`ensure`), evidence says the promise was not met\n\
         while its claims hold. The candidates are then its text, the word\n\
         that the claims say what the text says, what it requires and the\n\
         evidence, not the code: write the claim that would have caught it.\n\n\
         To fix: correct the code and record new evidence, or correct the\n\
         evidence if it is the one that is wrong.",
    ),
    (
        "M002",
        "evidence denies an assumption",
        "A `uniform`, `represented` or `uses` claim can't be checked by Mitra:\n\
         evidence vouches for it or denies it. Here a piece of evidence denies it, so\n\
         nothing rests on it any more.\n\n\
         To fix: correct or remove the claim, or correct the evidence.",
    ),
    (
        "M003",
        "two pieces of evidence contradict each other",
        "On the same input, one piece of evidence says the function passes and\n\
         another says it fails, or two pieces record two different outputs.\n\
         They can't both be true. Until this is settled, neither counts for\n\
         that input, and its class is not relied on.\n\n\
         A test that passes only sometimes shows up this way.\n\n\
         To fix: find out which one is right, and correct the other.",
    ),
    (
        "M004",
        "an assumption is contradicted by the evidence",
        "A `uniform` claim says a function behaves the same for every input of\n\
         a class: one passing input stands for the class. Here one input of a\n\
         class passes and another fails, so the claim can't hold together\n\
         with both results.\n\n\
         One of them is wrong: the claim (the class is too wide), the pass, or\n\
         the failure. Mitra ranks them from the trust given to each.\n\n\
         To fix: check the failing result. If it is real, split the class so\n\
         that the two inputs fall in different classes; then fix the code.",
    ),
    (
        "M005",
        "a measurement does not fit the other evidence",
        "A measure such as `rate(x in S, cond)` has a value either from a\n\
         measurement (`evidence m { for <claim> gives rate = 0.4 }`) or from\n\
         counting the observed outputs. Here a measurement disagrees with another\n\
         measurement of the same measure, or with what the observed outputs\n\
         leave possible.\n\n\
         To fix: find out which one is right, and correct the other.",
    ),
    (
        "M006",
        "evidence does not fit the model",
        "A piece of evidence gives a function inputs, or an output, that its\n\
         signature in the model does not allow. Either the observation is\n\
         wrong, or the model is: the code does take or return such a value.\n\
         The evidence judges nothing until this is settled.\n\n\
         To fix: if the observation is real, widen the type in the model;\n\
         if not, correct the evidence.",
    ),
    (
        "M007",
        "a claim contradicts the specs of what it calls",
        "A claim calls functions that have a body in the model. Computed from\n\
         those bodies, the claim is false on an input. The model says two\n\
         things that can't both be true, before any evidence.\n\n\
         To fix: correct the claim, or the body it contradicts.",
    ),
    (
        "M101",
        "a claim is not verified",
        "No evidence covers some inputs of the claim: nothing was tested\n\
         there, and no assumption lets a tested input stand for them.\n\n\
         A requirement (`ensure`) is not verified while its subject has no\n\
         claim.\n\n\
         To fix: add a test on the missing inputs, or state a `represented`\n\
         or `uniform` claim so that the tests you have stand for the others. `mitra next`\n\
         lists what would settle each claim.",
    ),
    (
        "M102",
        "a claim holds only under assumptions",
        "Every input of the claim is covered, but part of the cover rests on\n\
         claims that no evidence supports: a `uniform` or `represented` claim\n\
         nobody vouched for, an `assume`, the spec of a function it calls\n\
         that is not verified, or, for a requirement (`ensure`), a claim of\n\
         its subject, its capture or a requirement it names.\n\n\
         To fix: review each assumption and record the review as evidence\n\
         (`evidence r { for <claim> holds }`), or test until it is not needed.",
    ),
    (
        "M103",
        "an assumption is not vouched for",
        "A `uniform`, `represented` or `uses` claim can't be checked by Mitra.\n\
         Neither can the capture of a requirement (`<ensure>::captured`):\n\
         that the claims of a subject say what the text of its `ensure`\n\
         says. Until a piece of evidence vouches for it, what rests on it is\n\
         only assumed.\n\n\
         To fix: have someone review it against the code, or the claims\n\
         against the text, then record it:\n\
         `evidence r { for <claim> holds source \"review:<who>\" }`.",
    ),
    (
        "M104",
        "evidence that can't be used",
        "A piece of evidence is about a function Mitra computes: one with a\n\
         body, outside any subject. It is true by construction, and evidence\n\
         can neither support nor refute it. The entry is ignored.\n\n\
         To fix: record the evidence on the function under review (one of a\n\
         subject, or one without a body), or on a claim.",
    ),
    (
        "M105",
        "a requirement that nothing leans on",
        "`require need = other::promise;` says this subject leans on a promise\n\
         made elsewhere, and nothing declared beside the `require` shows it:\n\
         no function `uses` one declared beside that promise, directly or\n\
         through other functions, no body or claim calls one, and no\n\
         signature or record field names one of its types. The need is\n\
         stale, or the design is not built yet.\n\n\
         To fix: add the `uses` clause to the function that calls it, or\n\
         remove the `require`.",
    ),
    (
        "M106",
        "a function uses what its subject does not require",
        "A function `uses` a function of another subject or boundary, which\n\
         makes promises (`ensure`), and no `require` beside the function names\n\
         one of them: the code relies on something the intent does not say.\n\
         Only a subject or boundary that states an `ensure` or a `require`\n\
         is held to this. The same goes for a value taken by its contract,\n\
         `x: impl Contract` or a type parameter bound by one.\n\n\
         To fix: write `require <name> = other::promise;` beside the function.",
    ),
    (
        "M107",
        "evidence on a piece the model no longer has",
        "An entry of the ledger names a function or a claim that is not in\n\
         the model: it was removed or renamed. The entry is kept and judges\n\
         nothing. What a machine observed (a test, a tie) goes at its next\n\
         run; what a person said stays until someone decides.\n\n\
         To fix: `mitra remove --orphaned` lists these entries and removes\n\
         them; or say it again on the piece as it is named now, with\n\
         `mitra add`, then remove the old entry.",
    ),
    (
        "M108",
        "evidence said before its piece changed",
        "A person said this of a piece that reads otherwise today: the\n\
         entry keeps the fingerprint of the piece as it was (`seen`), and\n\
         the piece has another one now. The entry is kept and judges\n\
         nothing: a review of a claim does not carry over to the claim\n\
         reworded, and a bug reported on it may or may not still stand.\n\n\
         To fix: look at the piece again. If the entry still stands, say\n\
         it again with `mitra add`; then `mitra remove --stale <piece>`\n\
         removes the old one.",
    ),
];

/// The explanation of a code, for `mitra explain`.
pub fn explain(code: &str) -> Option<String> {
    let found = CODES.iter().find(|c| c.0.eq_ignore_ascii_case(code));
    found.map(|(code, headline, text)| format!("{code}: {headline}\n\n{text}\n"))
}
