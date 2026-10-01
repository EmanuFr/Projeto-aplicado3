use std::io::{self, Write};

// =====================================================================
// PARTE 1 — TIPOS USADOS PELAS DUAS FASES
// =====================================================================

#[derive(Clone, Debug)]
struct Paciente {
    cpf: u64, // CPF guardado como número: comparar números é mais rápido que comparar texto
    nome: String,
    nascimento: String,
    // Só a Fase 3 usa este campo:
    // Some(n) = está na fila e entrou no evento n;  None = não está na fila.
    chegada_na_fila: Option<u64>,
}

// Um paciente esperando na fila.
#[derive(Clone, Copy, Debug)]
struct NaFila {
    risco: u8,    // 1 = Emergência ... 5 = Não urgente
    chegada: u64, // número do evento em que entrou (vem do relógio do sistema)
    cpf: u64,
}

// Uma linha do relatório do dia (R7).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Atendimento {
    cpf: u64,
    risco: u8,
    espera: u64, // quantos eventos se passaram entre a entrada e a chamada
}

// Todos os "tratamentos" do enunciado viram um tipo de erro.
#[derive(Debug, PartialEq)]
enum Erro {
    CpfDuplicado,
    CpfNaoCadastrado,
    RiscoInvalido,
    JaEstaNaFila,
    FilaVazia,
    NaoEstaNaFila,
}

// A REGRA DE PRIORIDADE, usada nas duas fases:
// "a" deve ser chamado antes de "b"?
// Risco menor primeiro. Empate de risco → quem chegou antes.
fn vem_antes(a: &NaFila, b: &NaFila) -> bool {
    a.risco < b.risco || (a.risco == b.risco && a.chegada < b.chegada)
}

// Um "trait" é como uma interface: a lista de operações que todo sistema
// precisa ter. Fase 1 e Fase 3 implementam as mesmas operações, então o
// menu (parte 7) consegue usar qualquer uma das duas do mesmo jeito.
//
// Regra do relógio: toda operação de fila que dá certo (entrada, chamada,
// desistência) conta como 1 evento.
trait Sistema {
    fn cadastrar(&mut self, cpf: u64, nome: &str, nascimento: &str) -> Result<(), Erro>; // R1
    fn buscar_cadastro(&self, cpf: u64) -> Option<&Paciente>; // R2
    fn dar_entrada(&mut self, cpf: u64, risco: u8) -> Result<(), Erro>; // R3
    fn chamar_proximo(&mut self) -> Result<Atendimento, Erro>; // R4
    fn desistir(&mut self, cpf: u64) -> Result<(), Erro>; // R5
    fn tamanho_fila(&self) -> usize; // R6
    fn atendidos(&self) -> &Vec<Atendimento>; // usado pelo R7
}

// =====================================================================
// PARTE 2 — FASE 1: só vetores e busca linear
// =====================================================================

struct Fase1 {
    cadastros: Vec<Paciente>,
    fila: Vec<NaFila>, // NÃO ordenada
    atendidos: Vec<Atendimento>,
    relogio: u64,
}

impl Fase1 {
    fn new() -> Self {
        Fase1 { cadastros: Vec::new(), fila: Vec::new(), atendidos: Vec::new(), relogio: 0 }
    }

    // Procura o CPF na fila olhando um por um. O(q), q = tamanho da fila.
    fn posicao_na_fila(&self, cpf: u64) -> Option<usize> {
        for i in 0..self.fila.len() {
            if self.fila[i].cpf == cpf {
                return Some(i);
            }
        }
        None
    }
}

impl Sistema for Fase1 {
    // O(N): precisa olhar todos para saber se é duplicado.
    fn cadastrar(&mut self, cpf: u64, nome: &str, nascimento: &str) -> Result<(), Erro> {
        if self.buscar_cadastro(cpf).is_some() {
            return Err(Erro::CpfDuplicado);
        }
        self.cadastros.push(Paciente {
            cpf,
            nome: nome.to_string(),
            nascimento: nascimento.to_string(),
            chegada_na_fila: None,
        });
        Ok(())
    }

    // O(N): olha um por um até achar.
    fn buscar_cadastro(&self, cpf: u64) -> Option<&Paciente> {
        for p in &self.cadastros {
            if p.cpf == cpf {
                return Some(p);
            }
        }
        None
    }

    // O(N + q): busca o cadastro e depois olha a fila inteira.
    fn dar_entrada(&mut self, cpf: u64, risco: u8) -> Result<(), Erro> {
        if risco < 1 || risco > 5 {
            return Err(Erro::RiscoInvalido);
        }
        if self.buscar_cadastro(cpf).is_none() {
            return Err(Erro::CpfNaoCadastrado);
        }
        if self.posicao_na_fila(cpf).is_some() {
            return Err(Erro::JaEstaNaFila);
        }
        self.relogio += 1;
        self.fila.push(NaFila { risco, chegada: self.relogio, cpf });
        Ok(())
    }

    // O(q): olha a fila inteira procurando quem vem antes de todos.
    fn chamar_proximo(&mut self) -> Result<Atendimento, Erro> {
        if self.fila.is_empty() {
            return Err(Erro::FilaVazia);
        }
        let mut melhor = 0;
        for i in 1..self.fila.len() {
            if vem_antes(&self.fila[i], &self.fila[melhor]) {
                melhor = i;
            }
        }
        // swap_remove: coloca o último no lugar do removido. É O(1) e pode ser
        // usado porque a fila não é ordenada (a ordem vem da varredura acima).
        let p = self.fila.swap_remove(melhor);
        self.relogio += 1;
        let at = Atendimento { cpf: p.cpf, risco: p.risco, espera: self.relogio - p.chegada };
        self.atendidos.push(at);
        Ok(at)
    }

    // O(q)
    fn desistir(&mut self, cpf: u64) -> Result<(), Erro> {
        match self.posicao_na_fila(cpf) {
            None => Err(Erro::NaoEstaNaFila),
            Some(i) => {
                self.fila.swap_remove(i);
                self.relogio += 1;
                Ok(())
            }
        }
    }

    fn tamanho_fila(&self) -> usize {
        self.fila.len()
    }

    fn atendidos(&self) -> &Vec<Atendimento> {
        &self.atendidos
    }
}

// =====================================================================
// PARTE 3 — FASE 3, peça 1: TABELA HASH (encadeamento)
//
// Ideia: um vetor de "baldes". Cada balde é uma listinha (Vec).
// O CPF decide o balde:  balde = cpf % quantidade_de_baldes.
// Dois CPFs no mesmo balde = COLISÃO → os dois ficam na mesma listinha.
//
// Mantemos no máximo 1 paciente por balde, em média (fator de carga ≤ 1).
// Passou disso, dobramos o número de baldes e redistribuímos todos.
// Assim cada listinha tem, em média, ~1 elemento → busca O(1) ESPERADO.
// =====================================================================

struct TabelaHash {
    baldes: Vec<Vec<Paciente>>,
    quantidade: usize,
}

impl TabelaHash {
    fn new() -> Self {
        TabelaHash { baldes: vec![Vec::new(); 1024], quantidade: 0 }
    }

    // A função de dispersão.
    fn balde_do(&self, cpf: u64) -> usize {
        (cpf % self.baldes.len() as u64) as usize
    }

    fn buscar(&self, cpf: u64) -> Option<&Paciente> {
        let b = self.balde_do(cpf);
        for p in &self.baldes[b] {
            if p.cpf == cpf {
                return Some(p);
            }
        }
        None
    }

    // Igual à de cima, mas devolve o paciente de um jeito que permite alterá-lo.
    fn buscar_mut(&mut self, cpf: u64) -> Option<&mut Paciente> {
        let b = self.balde_do(cpf);
        for p in &mut self.baldes[b] {
            if p.cpf == cpf {
                return Some(p);
            }
        }
        None
    }

    // Devolve false se o CPF já existia.
    fn inserir(&mut self, p: Paciente) -> bool {
        if self.buscar(p.cpf).is_some() {
            return false;
        }
        if self.quantidade >= self.baldes.len() {
            self.crescer();
        }
        let b = self.balde_do(p.cpf);
        self.baldes[b].push(p);
        self.quantidade += 1;
        true
    }

    // Dobra o número de baldes. Custa O(N), mas acontece cada vez mais raramente
    // (com 1024, 2048, 4096...), então na média cada inserção custa O(1): AMORTIZADO.
    fn crescer(&mut self) {
        let nova_qtd = self.baldes.len() * 2;
        // take: pega o vetor antigo e deixa um vazio no lugar
        let antigos = std::mem::take(&mut self.baldes);
        self.baldes = vec![Vec::new(); nova_qtd];
        for balde in antigos {
            for p in balde {
                let b = self.balde_do(p.cpf); // o balde muda, porque o % mudou
                self.baldes[b].push(p);
            }
        }
    }
}

// =====================================================================
// PARTE 4 — FASE 3, peça 2: HEAP (fila de prioridade)
//
// É uma árvore guardada dentro de um vetor, sem ponteiros:
//     filhos de i: 2i+1 e 2i+2        pai de i: (i-1)/2
// Regra: o pai sempre "vem_antes" dos filhos → o próximo está em itens[0].
// A altura da árvore é log2(q) → inserir e remover custam O(log q).
// =====================================================================

struct Heap {
    itens: Vec<NaFila>,
}

impl Heap {
    fn new() -> Self {
        Heap { itens: Vec::new() }
    }

    // Coloca no fim e SOBE enquanto vier antes do pai.
    fn inserir(&mut self, x: NaFila) {
        self.itens.push(x);
        let mut i = self.itens.len() - 1;
        while i > 0 {
            let pai = (i - 1) / 2;
            if vem_antes(&self.itens[i], &self.itens[pai]) {
                self.itens.swap(i, pai);
                i = pai;
            } else {
                break;
            }
        }
    }

    // Tira o topo: troca com o último, remove, e faz o novo topo DESCER
    // trocando com o filho que vem antes, até ficar no lugar certo.
    fn remover_topo(&mut self) -> Option<NaFila> {
        if self.itens.is_empty() {
            return None;
        }
        let ultimo = self.itens.len() - 1;
        self.itens.swap(0, ultimo);
        let topo = self.itens.pop();

        let n = self.itens.len();
        let mut i = 0;
        loop {
            let esq = 2 * i + 1;
            let dir = 2 * i + 2;
            let mut escolhido = i;
            if esq < n && vem_antes(&self.itens[esq], &self.itens[escolhido]) {
                escolhido = esq;
            }
            if dir < n && vem_antes(&self.itens[dir], &self.itens[escolhido]) {
                escolhido = dir;
            }
            if escolhido == i {
                break; // os dois filhos vêm depois: está no lugar certo
            }
            self.itens.swap(i, escolhido);
            i = escolhido;
        }
        topo
    }
}

// =====================================================================
// PARTE 5 — FASE 3: junta hash + heap
//
// DESISTIR (R5) — escolhemos "marcar e ignorar" (a dica do enunciado):
//   - desistir só apaga a marca `chegada_na_fila` do paciente (O(1) esperado).
//     A entrada dele continua no heap, virando um "fantasma".
//   - quando um fantasma chega ao topo, chamar_proximo o descarta e tenta o próximo.
//   - Como saber se é fantasma? A entrada do heap guarda a chegada. Se ela não
//     bate com a `chegada_na_fila` atual do paciente, é fantasma. Isso resolve
//     até o caso de quem desiste e VOLTA: a entrada velha tem chegada diferente.
//   O que sacrifica: fantasmas ocupam memória no heap até saírem, e o tamanho
//   da fila precisa de um contador separado (o heap tem fantasmas dentro).
// =====================================================================

struct Fase3 {
    cadastros: TabelaHash,
    fila: Heap,
    pessoas_na_fila: usize, // não dá para usar o tamanho do heap: ele tem fantasmas
    atendidos: Vec<Atendimento>,
    relogio: u64,
}

impl Fase3 {
    fn new() -> Self {
        Fase3 {
            cadastros: TabelaHash::new(),
            fila: Heap::new(),
            pessoas_na_fila: 0,
            atendidos: Vec::new(),
            relogio: 0,
        }
    }
}

impl Sistema for Fase3 {
    // O(1) esperado e amortizado
    fn cadastrar(&mut self, cpf: u64, nome: &str, nascimento: &str) -> Result<(), Erro> {
        let p = Paciente {
            cpf,
            nome: nome.to_string(),
            nascimento: nascimento.to_string(),
            chegada_na_fila: None,
        };
        if self.cadastros.inserir(p) {
            Ok(())
        } else {
            Err(Erro::CpfDuplicado)
        }
    }

    // O(1) esperado
    fn buscar_cadastro(&self, cpf: u64) -> Option<&Paciente> {
        self.cadastros.buscar(cpf)
    }

    // O(1) esperado para achar o paciente + O(log q) para entrar no heap
    fn dar_entrada(&mut self, cpf: u64, risco: u8) -> Result<(), Erro> {
        if risco < 1 || risco > 5 {
            return Err(Erro::RiscoInvalido);
        }
        let p = match self.cadastros.buscar_mut(cpf) {
            None => return Err(Erro::CpfNaoCadastrado),
            Some(p) => p,
        };
        if p.chegada_na_fila.is_some() {
            return Err(Erro::JaEstaNaFila); // a marca responde isso em O(1)
        }
        self.relogio += 1;
        p.chegada_na_fila = Some(self.relogio);
        self.fila.inserir(NaFila { risco, chegada: self.relogio, cpf });
        self.pessoas_na_fila += 1;
        Ok(())
    }

    // O(log h) amortizado, h = tamanho do heap (incluindo fantasmas)
    fn chamar_proximo(&mut self) -> Result<Atendimento, Erro> {
        loop {
            let item = match self.fila.remover_topo() {
                None => return Err(Erro::FilaVazia),
                Some(item) => item,
            };
            let p = self.cadastros.buscar_mut(item.cpf).unwrap();
            if p.chegada_na_fila == Some(item.chegada) {
                // entrada válida: este é o paciente certo
                p.chegada_na_fila = None;
                self.pessoas_na_fila -= 1;
                self.relogio += 1;
                let at = Atendimento { cpf: item.cpf, risco: item.risco, espera: self.relogio - item.chegada };
                self.atendidos.push(at);
                return Ok(at);
            }
            // senão é fantasma de quem desistiu: ignora e o loop tenta o próximo
        }
    }

    // O(1) esperado
    fn desistir(&mut self, cpf: u64) -> Result<(), Erro> {
        let p = match self.cadastros.buscar_mut(cpf) {
            None => return Err(Erro::NaoEstaNaFila),
            Some(p) => p,
        };
        if p.chegada_na_fila.is_none() {
            return Err(Erro::NaoEstaNaFila);
        }
        p.chegada_na_fila = None; // vira fantasma no heap
        self.pessoas_na_fila -= 1;
        self.relogio += 1;
        Ok(())
    }

    fn tamanho_fila(&self) -> usize {
        self.pessoas_na_fila
    }

    fn atendidos(&self) -> &Vec<Atendimento> {
        &self.atendidos
    }
}

// =====================================================================
// PARTE 6 — ORDENAÇÃO DO RELATÓRIO (R7): maior espera primeiro
// =====================================================================

// Insertion sort: encaixa cada elemento na parte já ordenada à esquerda.
// O(n²). Estável: só passa na frente quem esperou ESTRITAMENTE mais (>).
fn insertion_sort(v: &mut Vec<Atendimento>) {
    for i in 1..v.len() {
        let x = v[i];
        let mut j = i;
        while j > 0 && x.espera > v[j - 1].espera {
            v[j] = v[j - 1]; // empurra para a direita
            j -= 1;
        }
        v[j] = x;
    }
}

// Merge sort: divide ao meio, ordena cada metade, intercala.
// Recorrência: T(n) = 2·T(n/2) + n  →  O(n log n). Estável também.
fn merge_sort(v: &[Atendimento]) -> Vec<Atendimento> {
    if v.len() <= 1 {
        return v.to_vec();
    }
    let meio = v.len() / 2;
    let esq = merge_sort(&v[..meio]); // 2·T(n/2)
    let dir = merge_sort(&v[meio..]);

    // intercala: O(n)
    let mut resultado = Vec::with_capacity(v.len());
    let mut i = 0;
    let mut j = 0;
    while i < esq.len() && j < dir.len() {
        if dir[j].espera > esq[i].espera {
            resultado.push(dir[j]);
            j += 1;
        } else {
            resultado.push(esq[i]); // empate: a esquerda primeiro → estável
            i += 1;
        }
    }
    while i < esq.len() {
        resultado.push(esq[i]);
        i += 1;
    }
    while j < dir.len() {
        resultado.push(dir[j]);
        j += 1;
    }
    resultado
}

fn relatorio_do_dia(s: &dyn Sistema, usar_merge: bool) -> Vec<Atendimento> {
    if usar_merge {
        merge_sort(s.atendidos())
    } else {
        let mut v = s.atendidos().clone();
        insertion_sort(&mut v);
        v
    }
}

// =====================================================================
// PARTE 7 — INTERFACE DE TERMINAL (fora do núcleo)
// =====================================================================

// Mostra a pergunta e lê uma linha digitada. Se a entrada acabar (Ctrl+D,
// ou fim de um arquivo redirecionado com `./main < comandos.txt`), encerra.
fn ler(pergunta: &str) -> String {
    print!("{pergunta}");
    io::stdout().flush().unwrap();
    let mut linha = String::new();
    match io::stdin().read_line(&mut linha) {
        Ok(0) | Err(_) => {
            println!("\nEntrada encerrada. Até logo!");
            std::process::exit(0);
        }
        Ok(_) => linha.trim().to_string(),
    }
}

// Aceita "123.456.789-09" ou "12345678909": fica só com os dígitos.
fn ler_cpf() -> Option<u64> {
    let texto = ler("CPF: ");
    let digitos: String = texto.chars().filter(|c| c.is_ascii_digit()).collect();
    if digitos.len() != 11 {
        println!("CPF inválido: digite os 11 números.");
        return None;
    }
    digitos.parse().ok()
}

fn formatar_cpf(cpf: u64) -> String {
    let s = format!("{cpf:011}");
    format!("{}.{}.{}-{}", &s[0..3], &s[3..6], &s[6..9], &s[9..11])
}

fn nome_do_risco(risco: u8) -> &'static str {
    match risco {
        1 => "Emergência",
        2 => "Muito urgente",
        3 => "Urgente",
        4 => "Pouco urgente",
        _ => "Não urgente",
    }
}

fn mensagem(e: &Erro) -> &'static str {
    match e {
        Erro::CpfDuplicado => "este CPF já está cadastrado.",
        Erro::CpfNaoCadastrado => "este CPF não tem cadastro. Cadastre o paciente primeiro (opção 1).",
        Erro::RiscoInvalido => "risco inválido: use um número de 1 a 5.",
        Erro::JaEstaNaFila => "este paciente já está na fila de espera.",
        Erro::FilaVazia => "a fila de espera está vazia.",
        Erro::NaoEstaNaFila => "este paciente não está na fila de espera.",
    }
}

// Gerador de números aleatórios simples (xorshift), usado só na opção 8.
struct Aleatorio(u64);

impl Aleatorio {
    fn proximo(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

fn opcao_cadastrar(s: &mut dyn Sistema) {
    let cpf = match ler_cpf() {
        Some(c) => c,
        None => return,
    };
    let nome = ler("Nome: ");
    let nascimento = ler("Data de nascimento (dd/mm/aaaa): ");
    match s.cadastrar(cpf, &nome, &nascimento) {
        Ok(()) => println!("Paciente cadastrado."),
        Err(e) => println!("Erro: {}", mensagem(&e)),
    }
}

fn opcao_buscar(s: &mut dyn Sistema) {
    let cpf = match ler_cpf() {
        Some(c) => c,
        None => return,
    };
    match s.buscar_cadastro(cpf) {
        Some(p) => println!("{} | CPF {} | nascimento {}", p.nome, formatar_cpf(p.cpf), p.nascimento),
        None => println!("Nenhum cadastro com esse CPF."),
    }
}

fn opcao_dar_entrada(s: &mut dyn Sistema) {
    let cpf = match ler_cpf() {
        Some(c) => c,
        None => return,
    };
    println!("Classificação de risco:");
    for r in 1..=5 {
        println!("  {r} - {}", nome_do_risco(r));
    }
    let risco: u8 = ler("Risco: ").parse().unwrap_or(0);
    match s.dar_entrada(cpf, risco) {
        Ok(()) => println!("Entrada registrada. Pessoas na fila: {}", s.tamanho_fila()),
        Err(e) => println!("Erro: {}", mensagem(&e)),
    }
}

fn opcao_chamar(s: &mut dyn Sistema) {
    match s.chamar_proximo() {
        Ok(at) => {
            let nome = match s.buscar_cadastro(at.cpf) {
                Some(p) => p.nome.clone(),
                None => String::from("?"),
            };
            println!(
                "Chamando: {} (CPF {}) | {} | esperou {} eventos",
                nome,
                formatar_cpf(at.cpf),
                nome_do_risco(at.risco),
                at.espera
            );
        }
        Err(e) => println!("Erro: {}", mensagem(&e)),
    }
}

fn opcao_desistir(s: &mut dyn Sistema) {
    let cpf = match ler_cpf() {
        Some(c) => c,
        None => return,
    };
    match s.desistir(cpf) {
        Ok(()) => println!("Paciente removido da fila."),
        Err(e) => println!("Erro: {}", mensagem(&e)),
    }
}

fn opcao_relatorio(s: &mut dyn Sistema) {
    let alg = ler("Ordenar com: 1 - insertion sort, 2 - merge sort: ");
    let usar_merge = alg == "2";
    let relatorio = relatorio_do_dia(s, usar_merge);
    if relatorio.is_empty() {
        println!("Nenhum paciente atendido ainda.");
        return;
    }
    let limite = ler(&format!("{} atendimentos. Quantas linhas mostrar? (Enter = todas): ", relatorio.len()));
    let limite: usize = limite.parse().unwrap_or(relatorio.len());

    println!("\n  #  | Espera | Risco         | CPF            | Nome");
    for (i, at) in relatorio.iter().take(limite).enumerate() {
        let nome = match s.buscar_cadastro(at.cpf) {
            Some(p) => p.nome.clone(),
            None => String::from("?"),
        };
        println!(
            "{:>4} | {:>6} | {:<13} | {} | {}",
            i + 1,
            at.espera,
            nome_do_risco(at.risco),
            formatar_cpf(at.cpf),
            nome
        );
    }
}

// Cadastra pacientes com CPFs aleatórios e, se pedido, coloca alguns na fila.
// Serve para testar o sistema com muitos dados sem digitar tudo.
fn opcao_gerar(s: &mut dyn Sistema) {
    let qtd: usize = ler("Quantos pacientes cadastrar? ").parse().unwrap_or(0);
    let na_fila: usize = ler("Quantos deles também entram na fila? ").parse().unwrap_or(0);
    let semente = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1)
        | 1; // o xorshift não pode começar em zero
    let mut rng = Aleatorio(semente);
    let mut cpfs = Vec::new();
    for i in 0..qtd {
        let cpf = rng.proximo() % 100_000_000_000;
        if s.cadastrar(cpf, &format!("Paciente {}", i + 1), "01/01/2000").is_ok() {
            cpfs.push(cpf);
        }
    }
    let mut entraram = 0;
    for &cpf in cpfs.iter().take(na_fila) {
        let risco = (rng.proximo() % 5) as u8 + 1;
        if s.dar_entrada(cpf, risco).is_ok() {
            entraram += 1;
        }
    }
    println!("{} pacientes cadastrados, {} colocados na fila.", cpfs.len(), entraram);
}

fn main() {
    println!("=== PRONTO ATENDIMENTO — TRIAGEM ===");
    println!("Qual versão usar?");
    println!("  1 - Fase 1 (listas simples)");
    println!("  3 - Fase 3 (tabela hash + heap)");

    // Box<dyn Sistema> = "uma caixa que guarda QUALQUER tipo que implemente
    // o trait Sistema". Assim o resto do programa não precisa saber qual é.
    let (mut sistema, nome_fase): (Box<dyn Sistema>, &str) = loop {
        match ler("Opção: ").as_str() {
            "1" => break (Box::new(Fase1::new()), "Fase 1"),
            "3" => break (Box::new(Fase3::new()), "Fase 3"),
            _ => println!("Digite 1 ou 3."),
        }
    };

    loop {
        println!("\n===== PRONTO ATENDIMENTO ({nome_fase}) =====");
        println!("1 - Cadastrar paciente");
        println!("2 - Buscar cadastro");
        println!("3 - Dar entrada na fila de espera");
        println!("4 - Chamar próximo paciente");
        println!("5 - Registrar desistência");
        println!("6 - Tamanho da fila");
        println!("7 - Relatório do dia");
        println!("8 - Gerar pacientes de teste");
        println!("9 - Medir desempenho (Fase 1 x Fase 3)");
        println!("0 - Sair");

        let s = sistema.as_mut();
        match ler("Opção: ").as_str() {
            "1" => opcao_cadastrar(s),
            "2" => opcao_buscar(s),
            "3" => opcao_dar_entrada(s),
            "4" => opcao_chamar(s),
            "5" => opcao_desistir(s),
            "6" => println!("Pacientes aguardando: {}", s.tamanho_fila()),
            "7" => opcao_relatorio(s),
            "8" => opcao_gerar(s),
            "9" => opcao_medir(),
               "0" => {
                println!("Até logo!");
                break;
            }
            _ => println!("Opção inválida."),
        }
    }
}

// =====================================================================
// PARTE 8 — MEDIÇÃO DE DESEMPENHO (Fase 2) — fora do núcleo
//
// O enunciado libera, FORA do núcleo, medição de tempo, aleatoriedade e
// estruturas prontas em código de teste. Por isso só esta parte usa
// HashMap/HashSet: elas servem para PREPARAR as operações e CONFERIR os
// resultados. O cronômetro mede apenas as chamadas a R1–R5 e às ordenações.
//
// Comparação justa: as duas fases recebem EXATAMENTE a mesma sequência de
// operações (mesma semente aleatória). Rodar de novo gera a mesma carga.
// =====================================================================

use std::collections::{HashMap, HashSet};
use std::hint::black_box;
use std::time::{Duration, Instant};

const SEMENTE: u64 = 2026;
const FILA_BASE: usize = 1000; // tamanho da sala de espera durante a medição
// Uma operação da Fase 3 dura ~20 ns, e ligar/desligar o cronômetro custa
// quase isso. Cronometrar uma por uma distorceria o resultado, então
// cronometramos LOTES de 100 operações e dividimos pelo total.
const LOTE: usize = 100;

// Do ponto de vista do TESTE (não do sistema): quem está na fila agora.
// Serve para sortear quem entra (alguém de fora) e quem desiste (alguém de dentro).
struct ControleFila {
    cpfs: Vec<u64>,
    posicao: HashMap<u64, usize>,
}

impl ControleFila {
    fn new() -> Self {
        ControleFila { cpfs: Vec::new(), posicao: HashMap::new() }
    }
    fn contem(&self, cpf: u64) -> bool {
        self.posicao.contains_key(&cpf)
    }
    fn entrar(&mut self, cpf: u64) {
        self.posicao.insert(cpf, self.cpfs.len());
        self.cpfs.push(cpf);
    }
    fn sair(&mut self, cpf: u64) {
        if let Some(i) = self.posicao.remove(&cpf) {
            self.cpfs.swap_remove(i);
            if i < self.cpfs.len() {
                let movido = self.cpfs[i];
                self.posicao.insert(movido, i);
            }
        }
    }
}

struct Resultado {
    ns_cadastrar: f64,
    ns_buscar: f64,
    ns_entrada: f64,
    ns_chamar: f64,
    ns_desistir: f64,
    total: Duration,
    erros: usize,
    atendidos: Vec<Atendimento>,
}

fn ns_por_op(d: Duration, quantidade: usize) -> f64 {
    d.as_nanos() as f64 / quantidade.max(1) as f64
}

// Sorteia LOTE pacientes que não estão na fila (e já os marca como dentro).
fn sortear_de_fora(cpfs: &[u64], controle: &mut ControleFila, rng: &mut Aleatorio) -> Vec<u64> {
    let mut escolhidos = Vec::with_capacity(LOTE);
    while escolhidos.len() < LOTE {
        let cpf = cpfs[(rng.proximo() % cpfs.len() as u64) as usize];
        if !controle.contem(cpf) {
            controle.entrar(cpf);
            escolhidos.push(cpf);
        }
    }
    escolhidos
}

fn riscos(rng: &mut Aleatorio) -> Vec<u8> {
    (0..LOTE).map(|_| (rng.proximo() % 5) as u8 + 1).collect()
}

// Sorteia um lote de quem está fora, cronometra as entradas e devolve
// (tempo gasto, quantidade de erros).
fn entradas_cronometradas(
    s: &mut dyn Sistema,
    cpfs: &[u64],
    controle: &mut ControleFila,
    rng: &mut Aleatorio,
) -> (Duration, usize) {
    let novos = sortear_de_fora(cpfs, controle, rng);
    let rs = riscos(rng);
    let mut erros = 0;
    let t = Instant::now();
    for k in 0..LOTE {
        if s.dar_entrada(novos[k], rs[k]).is_err() {
            erros += 1;
        }
    }
    (t.elapsed(), erros)
}

// Roda a carga completa em UM sistema (Fase 1 ou Fase 3).
fn medir_fase(s: &mut dyn Sistema, cpfs: &[u64], m: usize) -> Resultado {
    let inicio = Instant::now();
    let mut rng = Aleatorio(SEMENTE);
    let mut erros = 0;

    // ---- R1: cadastrar os N pacientes ----
    let t = Instant::now();
    for &cpf in cpfs {
        if s.cadastrar(cpf, "Paciente", "01/01/2000").is_err() {
            erros += 1;
        }
    }
    let d_cadastrar = t.elapsed();

    // ---- R2: M buscas de CPFs sorteados ----
    let consultas: Vec<u64> = (0..m).map(|_| cpfs[(rng.proximo() % cpfs.len() as u64) as usize]).collect();
    let mut achados = 0;
    let t = Instant::now();
    for &cpf in &consultas {
        if s.buscar_cadastro(cpf).is_some() {
            achados += 1;
        }
    }
    let d_buscar = t.elapsed();
    erros += m - black_box(achados);

    // ---- Enche a sala de espera (não cronometrado) ----
    let mut controle = ControleFila::new();
    while controle.cpfs.len() < FILA_BASE {
        let (_, e) = entradas_cronometradas(s, cpfs, &mut controle, &mut rng);
        erros += e;
    }

    // ---- R3, R5, R4: dia de movimento, em lotes ----
    // Cada lote: 100 entram, 100 desistem, 100 entram, 100 são chamados.
    // A fila fica sempre entre 1000 e 1100 pessoas. Quem desistiu pode ser
    // sorteado para entrar de novo, então o caso "desiste e volta" acontece.
    let (mut d_entrada, mut d_desistir, mut d_chamar) = (Duration::ZERO, Duration::ZERO, Duration::ZERO);
    let (mut n_entrada, mut n_desistir, mut n_chamar) = (0, 0, 0);
    let mut atendidos = Vec::with_capacity(m);
    let lotes = (m + LOTE - 1) / LOTE;

    for _ in 0..lotes {
        // 100 entram
        let (d, e) = entradas_cronometradas(s, cpfs, &mut controle, &mut rng);
        d_entrada += d;
        n_entrada += LOTE;
        erros += e;

        // 100 desistem (sorteados entre quem está esperando)
        let mut saem = Vec::with_capacity(LOTE);
        for _ in 0..LOTE {
            let i = (rng.proximo() % controle.cpfs.len() as u64) as usize;
            let cpf = controle.cpfs[i];
            controle.sair(cpf);
            saem.push(cpf);
        }
        let t = Instant::now();
        for &cpf in &saem {
            if s.desistir(cpf).is_err() {
                erros += 1;
            }
        }
        d_desistir += t.elapsed();
        n_desistir += LOTE;

        // mais 100 entram
        let (d, e) = entradas_cronometradas(s, cpfs, &mut controle, &mut rng);
        d_entrada += d;
        n_entrada += LOTE;
        erros += e;

        // chamadas
        let t = Instant::now();
        for _ in 0..LOTE {
            match s.chamar_proximo() {
                Ok(at) => atendidos.push(at),
                Err(_) => erros += 1,
            }
        }
        d_chamar += t.elapsed();
        n_chamar += LOTE;
        for at in &atendidos[atendidos.len() - LOTE..] {
            controle.sair(at.cpf);
        }
    }

    Resultado {
        ns_cadastrar: ns_por_op(d_cadastrar, cpfs.len()),
        ns_buscar: ns_por_op(d_buscar, m),
        ns_entrada: ns_por_op(d_entrada, n_entrada),
        ns_chamar: ns_por_op(d_chamar, n_chamar),
        ns_desistir: ns_por_op(d_desistir, n_desistir),
        total: inicio.elapsed(),
        erros,
        atendidos,
    }
}

fn opcao_medir() {
    println!("Mede as duas fases com a MESMA carga (não usa os dados do menu).");
    let n: usize = ler("N = pacientes cadastrados (ex.: 10000 ou 100000): ").parse().unwrap_or(0);
    let m: usize = ler("M = operações de cada tipo (ex.: 10000 ou 100000): ").parse().unwrap_or(0);
    if n < 3000 || m < LOTE {
        println!("Use N de pelo menos 3000 e M de pelo menos {LOTE}.");
        return;
    }

    // N CPFs diferentes, sorteados com semente fixa (11 dígitos).
    let mut rng = Aleatorio(SEMENTE ^ 0x9E37_79B9);
    let mut vistos = HashSet::new();
    let mut cpfs = Vec::with_capacity(n);
    while cpfs.len() < n {
        let cpf = 10_000_000_000 + rng.proximo() % 90_000_000_000;
        if vistos.insert(cpf) {
            cpfs.push(cpf);
        }
    }

    if n >= 100_000 {
        println!("Aviso: com N = {n}, a Fase 1 pode levar cerca de 1 minuto.");
    }
    print!("Medindo Fase 1... ");
    io::stdout().flush().unwrap();
    let r1 = medir_fase(&mut Fase1::new(), &cpfs, m);
    println!("{:.2} s", r1.total.as_secs_f64());
    print!("Medindo Fase 3... ");
    io::stdout().flush().unwrap();
    let r3 = medir_fase(&mut Fase3::new(), &cpfs, m);
    println!("{:.2} s", r3.total.as_secs_f64());

    // Ordenação dos M atendimentos (os mesmos nas duas fases).
    print!("Ordenando {} atendimentos... ", r3.atendidos.len());
    io::stdout().flush().unwrap();
    let mut v = r3.atendidos.clone();
    let t = Instant::now();
    insertion_sort(&mut v);
    let d_ins = t.elapsed();
    let t = Instant::now();
    let w = merge_sort(&r3.atendidos);
    let d_mer = t.elapsed();
    println!("ok");

    println!("\n=== RESULTADO: N = {n} cadastros, M = {m} operações ===");
    println!("{:<20} | {:>15} | {:>15} | {:>9}", "Operação", "Fase 1 (ns/op)", "Fase 3 (ns/op)", "F1 / F3");
    let linhas = [
        ("R1 cadastrar", r1.ns_cadastrar, r3.ns_cadastrar),
        ("R2 buscar_cadastro", r1.ns_buscar, r3.ns_buscar),
        ("R3 dar_entrada", r1.ns_entrada, r3.ns_entrada),
        ("R4 chamar_proximo", r1.ns_chamar, r3.ns_chamar),
        ("R5 desistir", r1.ns_desistir, r3.ns_desistir),
    ];
    for (nome, a, b) in linhas {
        println!("{:<20} | {:>15.1} | {:>15.1} | {:>8.0}x", nome, a, b, a / b);
    }

    let ms_ins = d_ins.as_secs_f64() * 1000.0;
    let ms_mer = d_mer.as_secs_f64() * 1000.0;
    println!("\nOrdenação do relatório ({} atendimentos):", v.len());
    println!("  insertion sort: {ms_ins:.2} ms | merge sort: {ms_mer:.2} ms | merge {:.0}x mais rápido", ms_ins / ms_mer);

    println!("\nConferências:");
    println!("  erros nas operações: Fase 1 = {}, Fase 3 = {}", r1.erros, r3.erros);
    println!("  as duas fases chamaram os mesmos pacientes na mesma ordem: {}", if r1.atendidos == r3.atendidos { "sim" } else { "NÃO" });
    println!("  insertion e merge deram o mesmo relatório: {}", if v == w { "sim" } else { "NÃO" });

    // Linhas prontas para colar numa planilha (separadas por ponto e vírgula).
    println!("\nCSV;N;M;operacao;fase1_ns;fase3_ns");
    for (nome, a, b) in linhas {
        println!("CSV;{n};{m};{nome};{a:.1};{b:.1}");
    }
    println!("CSV;{n};{m};insertion_sort_ms;{ms_ins:.2};");
    println!("CSV;{n};{m};merge_sort_ms;{ms_mer:.2};");
}
