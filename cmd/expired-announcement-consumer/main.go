package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/adjust/rmq/v4"
	"github.com/stevenhansel/enchiridion-api/internal/config"
)

type SyncJobPayload struct {
	Operation string `json:"operation"`
	URL       string `json:"imageUrl"`
	Filename  string `json:"filename"`
}

type ExpirationJobPayload struct {
	URL            string `json:"imageUrl"`
	Filename       string `json:"filename"`
	DeviceID       int    `json:"deviceId"`
	ExpirationTime int    `json:"expirationTime"`
}

func getSyncQueueName(deviceID int) string {
	return fmt.Sprintf("sync-device-%d", deviceID)
}
func getExpirationQueueName() string {
	return "expiration-device"
}

const (
	prefetchLimit = 1000
	pollDuration  = 100 * time.Millisecond
	numConsumers  = 5

	reportBatchSize = 10000
	consumeDuration = time.Millisecond
	shouldLog       = true
)

func main() {
	var environment config.Environment

	flag.Var(
		&environment,
		"env",
		"application environment, could be either (development|staging|production)",
	)

	flag.Parse()

	config, err := config.New(environment)
	if err != nil {
		panic(err)
	}

	errChan := make(chan error, 10)
	go logErrors(errChan)

	connection, err := rmq.OpenConnection("consumer", "tcp", config.RedisQueueAddr, 1, errChan)
	if err != nil {
		panic(err)
	}

	fmt.Println("starting...")

	queue, err := connection.OpenQueue(getExpirationQueueName())
	if err != nil {
		panic(err)
	}

	if err := queue.StartConsuming(prefetchLimit, pollDuration); err != nil {
		panic(err)
	}

	for i := 0; i < numConsumers; i++ {
		name := fmt.Sprintf("consumer %d", i)
		if _, err := queue.AddConsumer(name, NewConsumer(i, connection)); err != nil {
			panic(err)
		}
	}

	signals := make(chan os.Signal, 1)
	signal.Notify(signals, syscall.SIGINT)
	defer signal.Stop(signals)

	<-signals // wait for signal
	go func() {
		<-signals // hard exit on second signal (in case shutdown gets stuck)
		os.Exit(1)
	}()

	<-connection.StopAllConsuming() // wait for all Consume() calls to finish
}

type Consumer struct {
	name   string
	count  int
	before time.Time
	conn   rmq.Connection
}

func rejectDelivery(delivery rmq.Delivery) {
	if err := delivery.Reject(); err != nil {
		debugf("failed to reject %s", err)
	} else {
		debugf("rejected successfully")
	}
}

func NewConsumer(tag int, conn rmq.Connection) *Consumer {
	return &Consumer{
		name:   fmt.Sprintf("consumer%d", tag),
		count:  0,
		before: time.Now(),
		conn:   conn,
	}
}

func (consumer *Consumer) Consume(delivery rmq.Delivery) {
	payload := delivery.Payload()
	debugf("start consume %s", payload)

	data := &ExpirationJobPayload{}
	err := json.Unmarshal([]byte(payload), data)
	if err != nil {
		debugf("unmarshalling failed")
		rejectDelivery(delivery)
	}

	go consumer.handleExpirationTime(delivery, data)
}

func (consumer *Consumer) handleExpirationTime(delivery rmq.Delivery, payload *ExpirationJobPayload) {
	duration := time.Duration(payload.ExpirationTime) * time.Second

	fmt.Println("START COUNTDOWN")

	time.AfterFunc(duration, func() {
		// TODO: handle expiration time, update status + publishing a message to the queue
		fmt.Println("START PROCESS AFTER COUNTDOWN")
		syncQueue, err := consumer.conn.OpenQueue(getSyncQueueName(payload.DeviceID))
		if err != nil {
			fmt.Println("err: ", err)
			return
		}

		syncJobPayload, err := json.Marshal(SyncJobPayload{
			Operation: "delete",
			URL:       "",
			Filename:  payload.Filename,
		})
		if err != nil {
			fmt.Println("err: ", err)
			return
		}

		if err := syncQueue.Publish(string(syncJobPayload)); err != nil {
			fmt.Println("err: ", err)
			return
		}
		fmt.Println("JOB PUBLISHED")

		consumer.count++
		if consumer.count%reportBatchSize == 0 {
			duration := time.Now().Sub(consumer.before)
			consumer.before = time.Now()
			perSecond := time.Second / (duration / reportBatchSize)
			log.Printf("%s consumed %d %s %d", consumer.name, consumer.count, payload, perSecond)
		}

		if consumer.count%reportBatchSize > 0 {
			if err := delivery.Ack(); err != nil {
				debugf("failed to ack %s: %s", payload, err)
			} else {
				debugf("acked %s", payload)
			}
		} else { // reject one per batch
			if err := delivery.Reject(); err != nil {
				debugf("failed to reject %s: %s", payload, err)
			} else {
				debugf("rejected %s", payload)
			}
		}
	})
}

func logErrors(errChan <-chan error) {
	for err := range errChan {
		switch err := err.(type) {
		case *rmq.HeartbeatError:
			if err.Count == rmq.HeartbeatErrorLimit {
				log.Print("heartbeat error (limit): ", err)
			} else {
				log.Print("heartbeat error: ", err)
			}
		case *rmq.ConsumeError:
			log.Print("consume error: ", err)
		case *rmq.DeliveryError:
			log.Print("delivery error: ", err.Delivery, err)
		default:
			log.Print("other error: ", err)
		}
	}
}

func debugf(format string, args ...interface{}) {
	if shouldLog {
		log.Printf(format, args...)
	}
}
