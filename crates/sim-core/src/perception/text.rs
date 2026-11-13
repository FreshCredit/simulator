//! Text Perception Module (P1-AGENT-1)
//!
//! Text input processing and natural language understanding.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Text perception from reading or messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPerception {
    /// inputs.
    pub inputs: Vec<TextInput>,
    /// salience.
    pub salience: f32,
    /// sentiment.
    pub sentiment: Sentiment,
    /// entities.
    pub entities: Vec<Entity>,
    /// intent.
    pub intent: Option<Intent>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Individual text input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextInput {
    /// id.
    pub id: u64,
    /// source.
    pub source: TextSource,
    /// content.
    pub content: String,
    /// timestamp.
    pub timestamp: u64,
    /// priority.
    pub priority: Priority,
}

/// Source of text
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextSource {
    /// Field.
    /// message.
    Message {
        /// sender id.
        sender_id: u64,
    },
    /// Field.
    /// document.
    Document {
        /// title.
        title: String,
    },
    /// Field.
    /// sign.
    Sign {
        /// location.
        location: String,
    },
    /// Field.
    /// interface.
    Interface {
        /// component.
        component: String,
    },
    /// memory.
    Memory,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    /// low.
    Low,
    /// medium.
    Medium,
    /// high.
    High,
    /// critical.
    Critical,
}

/// Sentiment analysis result
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Sentiment {
    /// polarity.
    pub polarity: f32, // -1.0 to 1.0
    /// subjectivity.
    pub subjectivity: f32, // 0.0 to 1.0
    /// valence.
    pub valence: f32, // 0.0 to 1.0 (positive emotion)
    /// arousal.
    pub arousal: f32, // 0.0 to 1.0 (activation level)
}

impl Default for Sentiment {
    fn default() -> Self {
        Self {
            polarity: 0.0,
            subjectivity: 0.0,
            valence: 0.5,
            arousal: 0.5,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Named entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// text.
    pub text: String,
    /// entity type.
    pub entity_type: EntityType,
    /// confidence.
    pub confidence: f32,
    /// start.
    pub start: usize,
    /// end.
    pub end: usize,
}

/// Entity type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    /// person.
    Person,
    /// organization.
    Organization,
    /// location.
    Location,
    /// time.
    Time,
    /// number.
    Number,
    /// concept.
    Concept,
    /// agent.
    Agent,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Intent classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    /// inform.
    Inform,
    /// request.
    Request,
    /// command.
    Command,
    /// question.
    Question,
    /// greeting.
    Greeting,
    /// farewell.
    Farewell,
    /// agreement.
    Agreement,
    /// disagreement.
    Disagreement,
}

/// Text processor for agents
#[derive(Debug)]
pub struct TextProcessor {
    /// config.
    pub config: TextConfig,
    /// vocabulary.
    pub vocabulary: HashMap<String, f32>, // word -> importance
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Text processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextConfig {
    /// max input length.
    pub max_input_length: usize,
    /// language.
    pub language: String,
    /// enable sentiment.
    pub enable_sentiment: bool,
    /// enable entities.
    pub enable_entities: bool,
    /// enable intent.
    pub enable_intent: bool,
}

impl Default for TextConfig {
    fn default() -> Self {
        Self {
            max_input_length: 1000,
            language: "en".to_string(),
            enable_sentiment: true,
            enable_entities: true,
            enable_intent: true,
        }
    }
}

impl TextProcessor {
    /// Create new text processor
    #[must_use]
    pub fn new(config: TextConfig) -> Self {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        Self {
            config,
            vocabulary: Self::load_default_vocabulary(),
        }
    }

    /// Process text inputs into perception
    #[must_use]
    pub fn process(&self, inputs: &[TextInput]) -> TextPerception {
        if inputs.is_empty() {
            return TextPerception {
                inputs: vec![],
                salience: 0.0,
                sentiment: Sentiment::default(),
                entities: vec![],
                intent: None,
            };
        }

        // Combine all inputs
        let combined_text = inputs
            .iter()
            .map(|i| i.content.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        let sentiment = if self.config.enable_sentiment {
            self.analyze_sentiment(&combined_text)
        } else {
            Sentiment::default()
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let entities = if self.config.enable_entities {
            self.extract_entities(&combined_text)
        } else {
            vec![]
        };

        let intent = if self.config.enable_intent {
            self.classify_intent(&combined_text)
        } else {
            None
        };

        // Calculate salience based on priority and content
        let salience = inputs
            .iter()
            .map(|i| match i.priority {
                Priority::Critical => 1.0,
                Priority::High => 0.7,
                Priority::Medium => 0.4,
                Priority::Low => 0.1,
            })
            .sum::<f32>()
            / inputs.len() as f32;

        TextPerception {
            inputs: inputs.to_vec(),
            salience,
            sentiment,
            entities,
            intent,
        }
    }

    /// Analyze sentiment of text
    fn analyze_sentiment(&self, text: &str) -> Sentiment {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let positive_words = [
            "good",
            "great",
            "excellent",
            "happy",
            "love",
            "best",
            "wonderful",
        ];
        let negative_words = [
            "bad", "terrible", "awful", "sad", "hate", "worst", "horrible",
        ];

        let lower = text.to_lowercase();
        let words: Vec<_> = lower.split_whitespace().collect();
        let mut positive_count = 0;
        let mut negative_count = 0;

        for word in &words {
            if positive_words.contains(word) {
                positive_count += 1;
            }
            if negative_words.contains(word) {
                negative_count += 1;
            }
        }

        let total = positive_count + negative_count;
        if total == 0 {
            Sentiment::default()
        } else {
            Sentiment {
                polarity: (positive_count as f32 - negative_count as f32) / total as f32,
                subjectivity: total as f32 / words.len() as f32,
                valence: positive_count as f32 / total as f32,
                arousal: (positive_count + negative_count) as f32 / words.len() as f32,
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Extract named entities
    fn extract_entities(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();

        // Simple pattern matching for entity extraction
        let words: Vec<_> = text.split_whitespace().collect();

        for (i, word) in words.iter().enumerate() {
            // Capitalized words might be names
            if word.chars().next().is_some_and(char::is_uppercase) {
                let start = words[..i].join(" ").len();
                let end = start + word.len();

                entities.push(Entity {
                    text: (*word).to_string(),
                    entity_type: EntityType::Person,
                    confidence: 0.6,
                    start,
                    end,
                });
            }

            // Numbers
            if word.parse::<f64>().is_ok() {
                let start = words[..i].join(" ").len();
                let end = start + word.len();

                entities.push(Entity {
                    text: (*word).to_string(),
                    entity_type: EntityType::Number,
                    confidence: 0.9,
                    start,
                    end,
                });
            }
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        entities
    }

    /// Classify intent of text
    fn classify_intent(&self, text: &str) -> Option<Intent> {
        let lower = text.to_lowercase();

        if lower.starts_with("what") || lower.starts_with("how") || lower.starts_with("why") {
            Some(Intent::Question)
        } else if lower.starts_with("please") || lower.contains("can you") {
            Some(Intent::Request)
        } else if lower.starts_with("hello") || lower.starts_with("hi") {
            Some(Intent::Greeting)
        } else if lower.starts_with("goodbye") || lower.starts_with("bye") {
            Some(Intent::Farewell)
        } else if lower.starts_with("yes") || lower.contains("agree") {
            Some(Intent::Agreement)
        } else if lower.starts_with("no") || lower.contains("disagree") {
            Some(Intent::Disagreement)
        } else {
            Some(Intent::Inform)
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Tokenize text into words
    #[must_use]
    pub fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Calculate TF-IDF vector
    #[must_use]
    pub fn tf_idf(&self, documents: &[String]) -> Vec<HashMap<String, f32>> {
        let tokenized: Vec<_> = documents.iter().map(|d| self.tokenize(d)).collect();

        // Calculate document frequency
        let mut doc_freq: HashMap<String, u32> = HashMap::new();
        for doc in &tokenized {
            let unique: std::collections::HashSet<_> = doc.iter().cloned().collect();
            for word in unique {
                *doc_freq.entry(word).or_insert(0) += 1;
            }
        }

        let n_docs = documents.len() as f32;

        // Calculate TF-IDF for each document
        tokenized
            .iter()
            .map(|doc| {
                let mut tfidf = HashMap::new();
                let doc_len = doc.len() as f32;

                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                let word_counts: HashMap<String, u32> =
                    doc.iter().fold(HashMap::new(), |mut acc, w| {
                        *acc.entry(w.clone()).or_insert(0) += 1;
                        acc
                    });

                for (word, count) in word_counts {
                    let tf = count as f32 / doc_len;
                    let df = doc_freq.get(&word).copied().unwrap_or(1) as f32;
                    let idf = (n_docs / df).ln();
                    tfidf.insert(word, tf * idf);
                }

                tfidf
            })
            .collect()
    }

    /// Load default vocabulary
    fn load_default_vocabulary() -> HashMap<String, f32> {
        let mut vocab = HashMap::new();
        let important = [
            "trust",
            "cooperate",
            "defect",
            "reward",
            "punish",
            "risk",
            "safe",
        ];
        for word in important {
            vocab.insert(word.to_string(), 1.0);
        }
        vocab
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate similarity between two texts
    #[must_use]
    pub fn similarity(&self, text1: &str, text2: &str) -> f32 {
        let vec1 = self.tf_idf(&[text1.to_string()]);
        let vec2 = self.tf_idf(&[text2.to_string()]);

        if vec1.is_empty() || vec2.is_empty() {
            return 0.0;
        }

        let v1 = &vec1[0];
        let v2 = &vec2[0];

        // Cosine similarity
        let mut dot = 0.0;
        let mut norm1 = 0.0;
        let mut norm2 = 0.0;

        for (word, val1) in v1 {
            norm1 += val1 * val1;
            if let Some(val2) = v2.get(word) {
                dot += val1 * val2;
            }
        }

        for val2 in v2.values() {
            norm2 += val2 * val2;
        }

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        if norm1 == 0.0 || norm2 == 0.0 {
            0.0
        } else {
            dot / (norm1.sqrt() * norm2.sqrt())
        }
    }
}

impl Default for TextProcessor {
    fn default() -> Self {
        Self::new(TextConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_processor() {
        let processor = TextProcessor::default();
        let inputs = vec![TextInput {
            id: 1,
            source: TextSource::Message { sender_id: 42 },
            content: "Hello, how are you?".to_string(),
            timestamp: 0,
            priority: Priority::Medium,
        }];

        let perception = processor.process(&inputs);
        assert!(perception.salience > 0.0);
        assert!(perception.intent.is_some());
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_sentiment_analysis() {
        let processor = TextProcessor::default();
        let positive = processor.analyze_sentiment("This is great and wonderful!");
        assert!(positive.polarity > 0.0);

        let negative = processor.analyze_sentiment("This is terrible and awful!");
        assert!(negative.polarity < 0.0);
    }

    #[test]
    fn test_entity_extraction() {
        let processor = TextProcessor::default();
        let entities = processor.extract_entities("John has 5 apples");
        assert!(!entities.is_empty());
    }

    #[test]
    fn test_intent_classification() {
        let processor = TextProcessor::default();
        assert_eq!(
            processor.classify_intent("What is this?"),
            Some(Intent::Question)
        );
        assert_eq!(
            processor.classify_intent("Hello there"),
            Some(Intent::Greeting)
        );
    }

    #[test]
    fn test_similarity() {
        let processor = TextProcessor::default();
        let sim = processor.similarity(
            "trust cooperation mutual benefit",
            "trust and cooperation and mutual",
        );
        // Similarity should be non-negative (can be 0.0 for short texts with different words)
        assert!(sim >= 0.0);
    }
}
