package main

import (
	"fmt"
	"math/rand"
)

type DurlandPlayerAction string

const (
	Zumbaling DurlandPlayerAction = "Zumbaling"
	Gulboning DurlandPlayerAction = "Gulboning"
	Schlaming DurlandPlayerAction = "Schlaming"
)

type DurlandRaceName string
type DurlandRace interface {
	Name() DurlandRaceName
	// onAction(*DurlandStrategyAction)
}

type DurlandNationName string
type DurlandNation interface {
	Name() DurlandNationName
	onAction(*DurlandState, *DurlandStrategyAction)
}

type DurlandPlayer struct {
	Race   DurlandRace
	Nation DurlandNation

	Health       float32
	Money        float32
	Satisfaction float32
}

type DurlandAnimalName string
type DurlandAnimal struct {
	Name DurlandAnimalName
}

type DurlandBiomeType interface {
	Name() string
	onAction(*DurlandState, *DurlandStrategyAction)
}

type DurlandBiome struct {
	Type    DurlandBiomeType
	Animals []DurlandAnimal
}

type DurlandLocationName string
type DurlandLocation struct {
	Name   DurlandLocationName
	Biomes []DurlandBiome
}

type DurlandState struct {
	Locations []DurlandLocation

	Player       DurlandPlayer
	CurrentBiome *DurlandBiome

	CurrentBiomeTicks int
	BiomesHistory     []*DurlandBiome

	Ticks int
}

type DurlandStrategyMove struct {
	Biome *DurlandBiome
}

type DurlandStrategyAction struct {
	Action *DurlandPlayerAction

	HealthChange       float32
	MoneyChange        float32
	SatisfactionChange float32
}

type DurlandTickResult string

const (
	Ok   DurlandTickResult = "Ok"
	Died DurlandTickResult = "Died"
)

func (state *DurlandState) tick(strategy DurlandStrategy) DurlandTickResult {
	move := strategy.DecideMove(state)
	if move != nil {
		state.CurrentBiome = move.Biome
		state.BiomesHistory = append(state.BiomesHistory, move.Biome)
		state.CurrentBiomeTicks = 0
	} else {
		state.CurrentBiomeTicks += 1
	}

	action := strategy.DecideAction(state)
	state.onAction(&action)
	state.performAction(&action)
	tickResult := state.checkResult()

	state.Ticks += 1

	return tickResult
}

func (state *DurlandState) checkResult() DurlandTickResult {
	if state.Player.Health <= 0 || state.Player.Money <= 0 || state.Player.Satisfaction <= 0 {
		return Died
	}
	return Ok
}

func (state *DurlandState) onAction(action *DurlandStrategyAction) {
	state.Player.Nation.onAction(state, action)
	state.CurrentBiome.Type.onAction(state, action)
}

func (state *DurlandState) performAction(action *DurlandStrategyAction) {
	state.Player.Health += action.HealthChange
	state.Player.Money += action.MoneyChange
	state.Player.Satisfaction += action.SatisfactionChange
}

type DurlandStrategy interface {
	DecideMove(state *DurlandState) *DurlandStrategyMove
	DecideAction(state *DurlandState) DurlandStrategyAction
}

// Расы
type DurlandRaceShlendrics struct{}

const Shlendrics DurlandRaceName = "Шлендрики"

func (race DurlandRaceShlendrics) Name() DurlandRaceName {
	return Shlendrics
}

const Hipstics DurlandRaceName = "Хипстики"

type DurlandRaceHipstics struct{}

func (race DurlandRaceHipstics) Name() DurlandRaceName {
	return Hipstics
}

type DurlandRaceScufics struct{}

const Skufics DurlandRaceName = "Скуфики"

func (race DurlandRaceScufics) Name() DurlandRaceName {
	return Skufics
}

// Животные
const (
	Slesanders  DurlandAnimalName = "Слесандры"
	Sisanders   DurlandAnimalName = "Сисяндры"
	Chuchunders DurlandAnimalName = "Чучундры"
)

// Народы
// Шлендрики
type DurlandNationMozhors struct{}

const DurlandNationNameMozhors = "Можоры"

func (nation DurlandNationMozhors) Name() DurlandNationName {
	return DurlandNationNameMozhors
}
func (nation DurlandNationMozhors) onAction(state *DurlandState, action *DurlandStrategyAction) {
	if action.Action != nil && *action.Action == Gulboning {
		action.MoneyChange *= 1.23
	}
	if action.Action != nil && *action.Action == Zumbaling {
		if rand.Float32() < 0.33 {
			action.HealthChange = 0
		}
	}
}

type DurlandNationNisheborods struct{}

const DurlandNationNameNisheborods = "Нищебороды"

func (nation DurlandNationNisheborods) Name() DurlandNationName {
	return DurlandNationNameNisheborods
}
func (nation DurlandNationNisheborods) onAction(state *DurlandState, action *DurlandStrategyAction) {
	if action.Action != nil && *action.Action == Gulboning {
		action.MoneyChange *= 1.0 - 0.87
		action.HealthChange *= 1.76
	}
}

// Хипстики
type DurlandNationSoevs struct{}

const DurlandNationNameSoevs = "Соевые"

func (nation DurlandNationSoevs) Name() DurlandNationName {
	return DurlandNationNameSoevs
}
func (nation DurlandNationSoevs) onAction(state *DurlandState, action *DurlandStrategyAction) {
	for _, animal := range state.CurrentBiome.Animals {
		if animal.Name == Chuchunders {
			action.HealthChange -= 0.12
		}
	}
}

type DurlandNationProsvelens struct{}

const DurlandNationNameProsvelens = "Просветлённые"

func (nation DurlandNationProsvelens) Name() DurlandNationName {
	return DurlandNationNameProsvelens
}
func (nation DurlandNationProsvelens) onAction(state *DurlandState, action *DurlandStrategyAction) {
	if action.Action != nil && *action.Action == Schlaming {
		sisandrs := 0
		for _, biome := range state.BiomesHistory[max(0, len(state.BiomesHistory)-3):len(state.BiomesHistory)] {
			for _, animal := range biome.Animals {
				if animal.Name == Sisanders {
					sisandrs += 1
				}
			}
		}

		action.SatisfactionChange += 0.31 * float32(sisandrs)
	}
}

// Скуфики
type DurlandNationDroncents struct{}

const DurlandNationNameDroncents = "Дроценты"

func (nation DurlandNationDroncents) Name() DurlandNationName {
	return DurlandNationNameDroncents
}
func (nation DurlandNationDroncents) onAction(state *DurlandState, action *DurlandStrategyAction) {
	if action.Action != nil && *action.Action == Gulboning {
		action.HealthChange *= 0.5
		action.MoneyChange *= 0.5
		action.SatisfactionChange *= 0.5
	}
}

type DurlandNationZheleznouhs struct{}

const DurlandNationNameZheleznouhs = "Железноухие"

func (nation DurlandNationZheleznouhs) Name() DurlandNationName {
	return "Железноухие"
}
func (nation DurlandNationZheleznouhs) onAction(state *DurlandState, action *DurlandStrategyAction) {
	if action.Action != nil && *action.Action == Zumbaling {
		action.SatisfactionChange = 0
		if rand.Float32() < 0.33 {
			action.MoneyChange = 0
		}
	}
}

// Локации
// Воркленд
type DurlandBiomeBalbesburg struct{}

func (biome DurlandBiomeBalbesburg) Name() string {
	return "Балбесбург"
}
func (biome DurlandBiomeBalbesburg) onAction(state *DurlandState, action *DurlandStrategyAction) {
	if rand.Float32() < 0.15 {
		for _, animal := range state.CurrentBiome.Animals {
			if animal.Name == Slesanders {
				action.HealthChange -= 0.1
			}
		}
	}
}

type DurlandBiomeDolbesburg struct{}

func (biome DurlandBiomeDolbesburg) Name() string {
	return "Долбесбург"
}
func (biome DurlandBiomeDolbesburg) onAction(state *DurlandState, action *DurlandStrategyAction) {
	if action.Action != nil && *action.Action == Zumbaling {
		action.MoneyChange *= 1.2
		action.SatisfactionChange *= 1.3
	}
}

// Бичленд
type DurlandBiomeKuramarubs struct{}

func (biome DurlandBiomeKuramarubs) Name() string {
	return "Курамарибы"
}
func (biome DurlandBiomeKuramarubs) onAction(state *DurlandState, action *DurlandStrategyAction) {
	// TODO
}

type DurlandBiomePuntaPelicana struct{}

func (biome DurlandBiomePuntaPelicana) Name() string {
	return "Пунта-пеликана"
}
func (biome DurlandBiomePuntaPelicana) onAction(state *DurlandState, action *DurlandStrategyAction) {
	// TODO
}

// Праналенд
type DurlandBiomeShrinavans struct{}

func (biome DurlandBiomeShrinavans) Name() string {
	return "Шринаванс"
}
func (biome DurlandBiomeShrinavans) onAction(state *DurlandState, action *DurlandStrategyAction) {
	if action.Action != nil && *action.Action == Schlaming {
		action.HealthChange *= 1.13
	}
}

type BiomePuntaHareKirishi struct{}

func (biome BiomePuntaHareKirishi) Name() string {
	return "Харе-Кириши"
}
func (biome BiomePuntaHareKirishi) onAction(state *DurlandState, action *DurlandStrategyAction) {
	if state.Player.Nation.Name() == "Дроценты" {
		action.HealthChange -= state.Player.Health * 0.1
	}
}

func BuildDurlandState() DurlandState {
	locations := [...]DurlandLocation{
		{
			Name: "Воркленд",
			Biomes: []DurlandBiome{
				{
					Type: DurlandBiomeBalbesburg{},
					Animals: []DurlandAnimal{
						{Name: Slesanders},
						{Name: Slesanders},
						{Name: Slesanders},
						{Name: Sisanders},
						{Name: Chuchunders},
					},
				},
				{
					Type: DurlandBiomeDolbesburg{},
					Animals: []DurlandAnimal{
						{Name: Slesanders},
						{Name: Slesanders},
						{Name: Slesanders},
						{Name: Sisanders},
						{Name: Chuchunders},
					},
				},
			},
		},
		{
			Name: "Бичленд",
			Biomes: []DurlandBiome{
				{
					Type: DurlandBiomeKuramarubs{},
					Animals: []DurlandAnimal{
						{Name: Slesanders},
						{Name: Sisanders},
						{Name: Sisanders},
						{Name: Sisanders},
						{Name: Chuchunders},
					},
				},
				{
					Type: DurlandBiomePuntaPelicana{},
					Animals: []DurlandAnimal{
						{Name: Slesanders},
						{Name: Sisanders},
						{Name: Sisanders},
						{Name: Sisanders},
						{Name: Chuchunders},
					},
				},
			},
		},
		{
			Name: "Праналенд",
			Biomes: []DurlandBiome{
				{
					Type: DurlandBiomeShrinavans{},
					Animals: []DurlandAnimal{
						{Name: Slesanders},
						{Name: Sisanders},
						{Name: Chuchunders},
						{Name: Chuchunders},
						{Name: Chuchunders},
					},
				},
				{
					Type: BiomePuntaHareKirishi{},
					Animals: []DurlandAnimal{
						{Name: Slesanders},
						{Name: Sisanders},
						{Name: Chuchunders},
						{Name: Chuchunders},
						{Name: Chuchunders},
					},
				},
			},
		},
	}
	player := DurlandPlayer{
		Race:   DurlandRaceShlendrics{},
		Nation: DurlandNationMozhors{},

		Health:       10,
		Money:        10,
		Satisfaction: 10,
	}
	currentBiome := &locations[0].Biomes[0]

	state := DurlandState{
		Locations:         locations[:],
		Player:            player,
		CurrentBiome:      currentBiome,
		CurrentBiomeTicks: 0,
		BiomesHistory:     make([]*DurlandBiome, 0),
		Ticks:             0,
	}
	return state
}

type DurlandStrategyIdle struct{}

func (stategy *DurlandStrategyIdle) DecideMove(state *DurlandState) *DurlandStrategyMove {
	return nil
}
func (stategy *DurlandStrategyIdle) DecideAction(state *DurlandState) DurlandStrategyAction {
	return DurlandStrategyAction{
		Action:             nil,
		HealthChange:       -0.5,
		MoneyChange:        -0.5,
		SatisfactionChange: -0.5,
	}
}

func main() {
	state := BuildDurlandState()
	strategy := DurlandStrategyIdle{}

	for {
		tickResult := state.tick(&strategy)
		if tickResult == Ok {
			fmt.Printf("tick: %.4d Ok\n", state.Ticks)
		} else if tickResult == Died {
			fmt.Printf("tick: %.4d Died\n", state.Ticks)
			break
		}
	}

}
